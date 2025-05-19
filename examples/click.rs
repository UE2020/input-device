use input_device::{InputSimulator, SimulationError};

pub fn main() -> Result<(), SimulationError> {
    let mut sim = InputSimulator::new()?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("Testing click");
    for _ in 0..20 {
        sim.left_mouse_down()?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        sim.left_mouse_up()?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}
