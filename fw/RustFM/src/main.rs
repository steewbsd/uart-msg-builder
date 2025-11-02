#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{AnyPin, Level, Output, Speed},
    i2c::{self, I2c, Master},
    mode::Async,
    rcc::{self},
    Config, Peri,
};
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};

use mpu6050_dmp::{
    calibration::CalibrationParameters, quaternion::Quaternion, sensor_async::Mpu6050,
    yaw_pitch_roll::YawPitchRoll,
};

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    config.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(12_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    let p = embassy_stm32::init(config);

    // let iic = I2c::new_blocking(p.I2C1, p.PA9, p.PA10, i2c::Config::default());
    let iic = I2c::new(
        p.I2C1,
        p.PA9,
        p.PA10,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH7,
        i2c::Config::default(),
    );
    let _lsb_pin = Output::new(p.PA11, Level::Low, Speed::Low);

    let ok_pin = p.PC14;
    let fail_pin = p.PC15;
    let txpin = p.PA6;

    let _rxen = Output::new(p.PB0, Level::High, Speed::Low);
    let _txen = Output::new(p.PB1, Level::Low, Speed::Low);

    core::mem::forget(_rxen);
    core::mem::forget(_txen);
    core::mem::forget(_lsb_pin);

    let imu_int = ExtiInput::new(p.PA12, p.EXTI12, embassy_stm32::gpio::Pull::Down);

    spawner.spawn(transmit(txpin.into())).unwrap();
    spawner.spawn(read_mpu(iic, imu_int.into())).unwrap();

    spawner.spawn(status_leds(ok_pin.into())).unwrap();
    Timer::after_millis(500).await;
    spawner.spawn(status_leds(fail_pin.into())).unwrap();
}

#[embassy_executor::task(pool_size = 2)]
async fn status_leds(p: Peri<'static, AnyPin>) {
    let mut led = Output::new(p, Level::Low, Speed::Low);

    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn transmit(p: Peri<'static, AnyPin>) {
    let mut txpin = Output::new(p, Level::Low, Speed::VeryHigh);
    txpin.set_low();
    Timer::after_millis(1).await;
    txpin.set_high();
    Timer::after_millis(1).await;
    txpin.set_low();

    loop {
        txpin.toggle();
        Timer::after_millis(1).await;
    }
}

#[embassy_executor::task]
async fn read_mpu(iic: I2c<'static, Async, Master>, mut ext: ExtiInput<'static>) {
    let mut mpu = Mpu6050::new(iic, mpu6050_dmp::address::Address::default())
        .await
        .unwrap();
    mpu.initialize_dmp(&mut Delay).await.unwrap();
    // Configure calibration parameters
    let calibration_params = CalibrationParameters::new(
        mpu6050_dmp::accel::AccelFullScale::G2,
        mpu6050_dmp::gyro::GyroFullScale::Deg2000,
        mpu6050_dmp::calibration::ReferenceGravity::ZN,
    );

    // info!("Calibrating Sensor");
    // mpu
    //     .calibrate(&mut Delay, &calibration_params)
    //     .await
    //     .unwrap();
    // info!("Sensor Calibrated");
    
    mpu.enable_dmp().await.unwrap();
    mpu.load_firmware().await.unwrap();
    mpu.boot_firmware().await.unwrap();
    // mpu.set_clock_source(mpu6050_dmp::clock_source::ClockSource::Xgyro).unwrap();
    let mut fifo: [u8; 28] = [0; 28];
    mpu.interrupt_fifo_oflow_en().await.unwrap();
    mpu.enable_fifo().await.unwrap();
    loop {
        ext.wait_for_rising_edge().await;
        // info!("Received new data");
        mpu.read_fifo(&mut fifo).await.unwrap();
        // let accel = mpu.gyro().unwrap();
        // trace!("X: {}, Y: {}, Z: {}" , accel.x(), accel.y(),accel.z());
        // trace!("{}", fifo);
        let quat = Quaternion::from_bytes(&fifo[..16]).unwrap().normalize();
        let ypr = YawPitchRoll::from(quat);
        let yaw_deg = ypr.yaw * 180.0 / core::f32::consts::PI;
        let pitch_deg = ypr.pitch * 180.0 / core::f32::consts::PI;
        let roll_deg = ypr.roll * 180.0 / core::f32::consts::PI;

        info!(
            "  Angles [deg]: yaw={}, pitch={}, roll={}",
            yaw_deg, pitch_deg, roll_deg
        );
        mpu.interrupt_read_clear().await.unwrap();
        mpu.reset_fifo().await.unwrap();
    }
}
