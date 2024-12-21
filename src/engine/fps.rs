pub struct FPSCounter
{
    last_60_frames : [f32; 60],
    size : usize,
    i : usize
}

impl FPSCounter
{
    pub fn new() -> Self
    {
        Self
        {
            last_60_frames : [0.; 60],
            size : 0,
            i : 0,
        }
    }

    pub fn add_frametime(&mut self, dt : f32)
    {
        self.last_60_frames[self.i % 60] = dt;
        self.i += 1;
        if self.size < 60
        {
            self.size += 1;
        }
    }

    pub fn get_fps(&self) -> u32
    {
        let sum : f32 = self.last_60_frames.iter().sum();
        let fps = 1. / (sum / self.size.max(1) as f32);

        fps.ceil() as u32
    }
}