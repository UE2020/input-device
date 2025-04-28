use input_device::{InputSimulator, SimulationError};

pub fn main() -> Result<(), SimulationError> {
    let mut sim = InputSimulator::new()?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    let (width, height) = sim.get_screen_size()?;
    let (center_x, center_y) = (width / 2, height / 2);
    for theta in 0..(std::f64::consts::PI * 100.0 * 30.0) as i32 {
        let theta = theta as f64 / 100.0;
        let x = 2.0 * theta * theta.cos() + center_x as f64;
        let y = 2.0 * theta * theta.sin() + center_y as f64;
        sim.pen(x as i32, y as i32, 0.5, 0, 0)?;
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
    sim.pen(0, 0, 0.0, 0, 0)?;
    Ok(())
}
