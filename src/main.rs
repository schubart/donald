#![warn(clippy::all)]

pub mod robot;

fn main() {
    let mut robot = robot::Robot::new();

    // Start the toy, calibrate robot while the toy flashes all lights.
    robot.lower_all_hands();
    robot.calibrate(std::time::Duration::from_millis(1300));
    robot.lift_all_hands();

    // Keep track of a growing sequence of colors.
    let mut colors = Vec::new();

    loop {
        // Expect the toy to flash the sequence so far.
        for &color in &colors {
            robot.wait_for_light_on(color);
            robot.wait_for_light_off(color);
        }

        // Expect the toy to flash one new light.
        let color = robot.wait_for_any_light_on();
        robot.wait_for_light_off(color);
        colors.push(color);

        // Move hands in the sequence shown by the toy.
        for &color in &colors {
            robot.lower_hand(color);
            robot.wait_for_light_on(color);
            robot.lift_hand(color);
            robot.wait_for_light_off(color);
        }

        // The toy stops after a sequence of 100 colors: Victory.
        if colors.len() == 100 {
            break;
        }
    }
}
