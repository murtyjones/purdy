use core::slice::Iter;

#[derive(Clone, Debug, PartialEq)]
pub struct DashPattern {
    array: Vec<f32>,
    phase: f32,
}

impl Default for DashPattern {
    fn default() -> Self {
        DashPattern {
            array: vec![],
            phase: 0.0,
        }
    }
}

impl DashPattern {
    pub fn new(array: Vec<f32>, phase: f32) -> Self {
        DashPattern { array, phase }
    }

    pub fn iter(&self) -> Iter<'_, f32> {
        self.array.iter()
    }
}
