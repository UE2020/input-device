use input_device::{InputSimulator, SimulationError};

pub fn main() -> Result<(), SimulationError> {
    let mut sim = InputSimulator::new()?;
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
    println!("Testing scroll");
    for _ in 0..30 {
        sim.wheel(10, 0)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Testing click");
    sim.left_mouse_down()?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    sim.move_mouse_rel(0, 20)?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    sim.left_mouse_up()?;
    Ok(())
}
