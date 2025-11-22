use embassy_stm32::{
    exti::ExtiInput,
    i2c::{I2c, Master},
    mode::Async,
    usart::Uart,
};
use embassy_time::Delay;
use mpu6050_dmp::{
    calibration::CalibrationParameters, quaternion::Quaternion, sensor_async::Mpu6050,
    yaw_pitch_roll::YawPitchRoll,
};

use defmt::info;

// Holds the current body state
struct MotionState {
    // gyroscope readings
    yaw: f32,
    pitch: f32,
    roll: f32,
    // accelerometer readings
    accel_x: f32,
    accel_y: f32,
    accel_z: f32,
    // magnetometer readings
        
}


#[embassy_executor::task]
pub async fn read_mpu(
    iic: I2c<'static, Async, Master>,
    mut ext: ExtiInput<'static>,
    tmtry: Uart<'static, Async>,
) {
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
    mpu.set_sample_rate_divider(9).await.unwrap();
    mpu.set_digital_lowpass_filter(mpu6050_dmp::config::DigitalLowPassFilter::Filter1)
        .await
        .unwrap();

    // mpu.set_clock_source(mpu6050_dmp::clock_source::ClockSource::Xgyro).unwrap();
    let mut fifo: [u8; 28] = [0; 28];
    let mut databuf_yaw: [f32; 10] = [0.0; 10];
    let mut databuf_pitch: [f32; 10] = [0.0; 10];
    let mut databuf_roll: [f32; 10] = [0.0; 10];
    let mut i = 0;

    // set up the interrupt so we receive data from the internal mpu6050 dmp,
    // combining gyro and accel
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
        // write the big endian repr of the sensor angles to UART for
        // plot representation in the telemetry receiving device.
        // let yaw_bytes = yaw_deg.to_ne_bytes();
        // let pitch_bytes = pitch_deg.to_ne_bytes();
        // let roll_bytes = roll_deg.to_ne_bytes();

        // tmtry.blocking_write(&yaw_bytes).unwrap();
        // tmtry.blocking_write(&pitch_bytes).unwrap();
        // tmtry.blocking_write(&roll_bytes).unwrap();
        // tmtry.write(&[255]).await.unwrap();

        // info!(
        //     "  Angles [deg]: yaw={}, pitch={}, roll={}",
        //     yaw_deg, pitch_deg, roll_deg
        // );
        info!(
            "  Angles [quat]: w={}, x={}, y={}, z={}",
            quat.w, quat.x, quat.y, quat.z
        );

        mpu.interrupt_read_clear().await.unwrap();
        mpu.reset_fifo().await.unwrap();
    }
}
