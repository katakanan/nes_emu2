use std::time::Instant;

#[macro_export]
macro_rules! yield_all {
    ($gen: expr) => {{
        let mut gen = $gen;
        loop {
            match std::pin::Pin::new(&mut gen).resume(()) {
                CoroutineState::Yielded(yielded) => {
                    yield yielded;
                }
                CoroutineState::Complete(result) => {
                    break result;
                }
            }
        }
    }};
}

pub struct FPS {
    pub last_tick: Instant,
}

impl FPS {
    pub fn new() -> Self {
        let last_tick = std::time::Instant::now();
        FPS { last_tick }
    }

    pub fn update(&mut self) -> String {
        let tmp = std::time::Instant::now();
        let duration = tmp - self.last_tick;
        self.last_tick = tmp;
        let fps = 1000.0 / (duration.as_millis() as f32);
        format!("fps:{:.*}", 1, fps)
    }
}
