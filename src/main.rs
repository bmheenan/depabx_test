use depabx::wrap;
use logger::Logger;
use std::fmt::Display;

// Assume this is our dependency; it simply exposes a concrete type. Frequently, we don't own this code, so we can't
// change it, and even if we did, it would be awkward and verbose to define a trait within this module.
mod logger {
    use std::fmt::Display;

    pub struct Logger;

    impl Logger {
        // We have some means of getting a concrete type
        pub fn new(_credentials: String) -> Self {
            Logger {}
        }

        // That type has methods that do work
        pub fn log_event<T: Display>(&self, description: T) {
            println!("Event: {description}");
        }

        // And more work
        pub fn log_error<T: Display>(&self, description: T) {
            println!("Error: {description}");
        }
    }
}

// Here we define an abstract version of our dependency:
// - Create a trait which should be named "AbxXxx" where Xxx is the name of the concrete type we're abstracting.
// - Define functions for each method we intend to use, prepending "abx_" to each method name.
// - Use #[wrap(Xxx)] on the concrete type to automatically implement the trait on that type.
#[wrap(Logger)]
trait AbxLogger {
    fn abx_log_event<T: Display>(&self, desription: T);
    fn abx_log_error<T: Display>(&self, desription: T);
}

// Set up when running in prod (and possibly integration tests). Keep this as minimal as possible as this is the part
// that doesn't get unit tested.
fn main() {
    // This is a good pattern. Simply call a function that contains all the logic to test, passing the real dependency.
    run(&Logger::new("prod credentials".to_string()));
}

// This is the code we unit test.
// Notice how this code doesn't know the concrete type it's using as its dependency. It instead uses the abstract trait
// AbxXxx with its abx_xxx methods. This code is easy to unit test by abstracting and injecting its dependencies.
fn run<L: AbxLogger>(logger: &L) {
    logger.abx_log_event("Some event description");
    logger.abx_log_error("Some error description");
}

#[cfg(test)]
mod tests {
    use super::{run, AbxLogger};
    use std::{cell::RefCell, fmt::Display};

    #[test]
    fn it_prints_two_logs() {
        // In our unit test, we quickly create a mock of our dependency by implementing the AbxXxx trait
        struct FakeLogger {
            logs: RefCell<Vec<String>>,
        }
        impl AbxLogger for FakeLogger {
            fn abx_log_event<T: Display>(&self, description: T) {
                self.logs.borrow_mut().push(description.to_string());
            }
            fn abx_log_error<T: Display>(&self, description: T) {
                self.logs.borrow_mut().push(description.to_string());
            }
        }
        impl FakeLogger {
            fn get_logs(&self) -> Vec<String> {
                self.logs.borrow().iter().map(|s| s.clone()).collect()
            }
        }

        let logger = FakeLogger {
            logs: RefCell::new(Vec::new()),
        };

        // Here, we invoke the code we wish to unit test, which accepts our mock just as easily as the original
        // dependency. It can't tell the difference.
        run(&logger);

        // This would have been hard to unit test if the code above was using the concrete Logger type. Since we were
        // able to pass it a mock, we can have that mock track what would have been logged and verify it here.
        let logs = logger.get_logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0], "Some event description");
        assert_eq!(logs[1], "Some error description");
    }
}
