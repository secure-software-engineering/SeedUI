use inputs_database::InputsDatabase;
use sut_database::SUT;

pub struct AppState {
    inputs_db: InputsDatabase,
    sut_db: SUT,
}

impl AppState {
    pub fn get_inputs_db(&self) -> &InputsDatabase {
        &self.inputs_db
    }

    pub fn get_sut_db(&self) -> &SUT {
        &self.sut_db
    }

    pub fn new(inputs: InputsDatabase, sut: SUT) -> Self {
        AppState {
            inputs_db: inputs.clone(),
            sut_db: sut.clone(),
        }
    }
}
