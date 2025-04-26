use input_device::{InputDeviceSimulator, SimulationError};

pub fn main() -> Result<(), SimulationError> {
    let mut sim = InputDeviceSimulator::new()?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("Testing absolute movement");
    for i in 0..30 {
        sim.move_mouse_abs(i * 10, i * 10)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    sim.move_mouse_abs(0, 0)?;
    println!("Testing relative movement");
    for _ in 0..30 {
        sim.move_mouse_rel(10, 10)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}
