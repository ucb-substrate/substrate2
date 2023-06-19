use std::{
    cell::{OnceCell, RefCell},
    ops::{Deref, DerefMut},
    thread::{self, JoinHandle},
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct Instance(usize);

pub struct InstancePtr {
    handle: RefCell<Option<JoinHandle<Instance>>>,
    inst: OnceCell<Instance>,
}

#[allow(clippy::new_ret_no_self)]
impl Instance {
    pub fn new(val: usize) -> InstancePtr {
        let handle = thread::spawn(move || {
            println!("Start generating instance {}", val);
            thread::sleep(Duration::from_secs(1));
            println!("Done generating instance {}", val);
            Instance(val)
        });

        InstancePtr {
            handle: RefCell::new(Some(handle)),
            inst: OnceCell::new(),
        }
    }

    pub fn increment(&mut self) {
        self.0 += 1
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

impl Clone for InstancePtr {
    fn clone(&self) -> Self {
        Self {
            handle: RefCell::new(None),
            inst: OnceCell::from((**self).clone()),
        }
    }
}

impl Deref for InstancePtr {
    type Target = Instance;

    fn deref(&self) -> &Self::Target {
        if let Some(inst) = self.inst.get() {
            inst
        } else {
            let handle = self.handle.take().unwrap();
            self.inst.set(handle.join().unwrap()).unwrap();
            self.inst.get().unwrap()
        }
    }
}

impl DerefMut for InstancePtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.inst.get().is_none() {
            let handle = self.handle.take().unwrap();
            self.inst.set(handle.join().unwrap()).unwrap();
        }
        self.inst.get_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::Instance;

    #[test]
    fn test_instance_generation() {
        let mut inst1 = Instance::new(1);
        let inst2 = Instance::new(2);

        assert_eq!(inst1.get(), 1);
        inst1.increment();
        assert_eq!(inst1.get(), 2);

        let inst3 = Instance::new(3);

        println!("Sleeping 2 seconds...");
        thread::sleep(Duration::from_secs(2));
        println!("Finished sleeping, all generation should be complete.");

        assert_eq!(inst2.get(), 2);
        assert_eq!(inst3.get(), 3);

        let inst4 = Instance::new(4);
        let mut inst5 = inst4.clone();

        assert_eq!(inst4.get(), inst5.get());
        inst5.increment();
        assert_eq!(inst4.get(), 4);
        assert_eq!(inst5.get(), 5);
    }
}
