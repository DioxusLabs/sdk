use crate::DioxusStdError;

pub struct CameraContext {
    ctx: nokhwa::Camera,
    stream_open: bool,
    resolution: (u32, u32),
}

impl CameraContext {
    pub fn stream(&mut self) -> Result<Vec<u8>, DioxusStdError> {
        self.open_stream()?;

        todo!()
        // TODO: Needs to return some kind of stream
    }

    fn open_stream(&mut self) -> Result<(), DioxusStdError> {
        if self.stream_open {
            return Ok(());
        }

        match self.ctx.open_stream() {
            Ok(()) => self.stream_open = true,
            Err(e) => return Err(DioxusStdError::Camera(e.to_string())),
        }

        // TODO: Needs to open a thread that continously updates a stream with the newest frame

        Ok(())
    }

    fn stop_stream(&mut self) -> Result<(), DioxusStdError> {
        if !self.stream_open {
            return Ok(());
        }

        // TODO: Needs to close the thread that is updating the frame

        match self.ctx.stop_stream() {
            Ok(()) => {
                self.stream_open = true;
                Ok(())
            }
            Err(e) => Err(DioxusStdError::Camera(e.to_string())),
        }
    }
}

pub struct CameraDevice {
    id: usize,
    name: String,
    description: String,
    misc_info: String,
}

pub enum FpsSelector {
    Camera,
    Custom(u32),
}

pub fn get_camera(
    device_id: usize,
    fps: Option<FpsSelector>,
) -> Result<CameraContext, DioxusStdError> {
    todo!("this feature is a work in progress")
}

pub fn get_camera_devices() -> Result<Vec<CameraDevice>, DioxusStdError> {
    todo!("this feature is a work in progress")
}
