use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use once_cell::sync::OnceCell;

#[derive(Default)]
pub struct Context {
    cells: HashMap<Params, Arc<OnceCell<Cell>>>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate(&mut self, params: Params) -> Instance {
        let cell = if let Some(cell) = self.cells.get(&params) {
            cell.clone()
        } else {
            let cell = Arc::new(OnceCell::new());

            self.cells.insert(params, cell.clone());

            let cell2 = cell.clone();

            thread::spawn(move || {
                println!("Generating cell with params {:?}", params);
                thread::sleep(Duration::from_secs(1));
                println!("Finished generating cell with params {:?}", params);
                cell2
                    .set(Cell {
                        width: params.width,
                        height: params.height,
                        area: params.width * params.height,
                    })
                    .unwrap();
            });

            cell
        };
        Instance { cell, loc: 0 }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Params {
    width: usize,
    height: usize,
}

#[derive(Debug)]
pub struct Cell {
    width: usize,
    height: usize,
    area: usize,
}

#[derive(Debug, Clone)]
pub struct Instance {
    cell: Arc<OnceCell<Cell>>,
    loc: usize,
}

impl Instance {
    pub fn cell(&self) -> &Cell {
        self.cell.wait()
    }

    pub fn move_right(&mut self, x: usize) {
        self.loc += x;
    }

    pub fn loc(&self) -> usize {
        self.loc
    }

    pub fn width(&self) -> usize {
        self.cell().width
    }

    pub fn height(&self) -> usize {
        self.cell().height
    }

    pub fn area(&self) -> usize {
        self.cell().area
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::{Context, Params};

    #[test]
    fn test_instance_generation() {
        let mut ctx = Context::new();

        let params1 = Params {
            width: 5,
            height: 10,
        };
        let mut inst1 = ctx.generate(params1);
        let mut inst2 = inst1.clone();
        let inst3 = ctx.generate(params1);

        inst1.move_right(5);

        let params2 = Params {
            width: 10,
            height: 7,
        };
        let inst4 = ctx.generate(params2);
        let inst5 = ctx.generate(params1);

        println!("Sleeping 2 seconds...");
        thread::sleep(Duration::from_secs(2));
        println!("Finished sleeping. All generation should have completed.");

        inst2.move_right(inst1.width());

        assert_eq!(inst1.area(), 50);
        assert_eq!(inst2.area(), 50);
        assert_eq!(inst3.area(), 50);
        assert_eq!(inst4.area(), 70);
        assert_eq!(inst5.area(), 50);

        assert_eq!(inst1.loc(), 5);
        assert_eq!(inst2.loc(), 5);
        assert_eq!(inst3.loc(), 0);
        assert_eq!(inst4.loc(), 0);
        assert_eq!(inst5.loc(), 0);
    }
}
