use input_device::{InputSimulator, SimulationError};

pub fn main() -> Result<(), SimulationError> {
    let mut sim = InputSimulator::new()?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    sim.touch_down(0, 200, 200).unwrap();
    sim.touch_down(1, 200, 500).unwrap();
    for i in 1..10 {
        sim.touch_move(0, 200, 200 + i * 10).unwrap();
        sim.touch_move(1, 200, 500 - i * 10).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    sim.touch_up(0)?;
    sim.touch_up(1)?;
    Ok(())
}
