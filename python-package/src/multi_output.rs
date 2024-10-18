use crate::utils::int_map_to_constraint_map;
use crate::utils::to_value_error;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use perpetual_rs::booster::MissingNodeTreatment;
use perpetual_rs::constraints::Constraint;
use perpetual_rs::data::Matrix;
use perpetual_rs::multi_output::MultiOutputBooster as CrateMultiOutputBooster;
use perpetual_rs::objective::Objective;
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyType;
use std::collections::{HashMap, HashSet};

#[pyclass(subclass)]
pub struct MultiOutputBooster {
    booster: CrateMultiOutputBooster,
}

#[pymethods]
impl MultiOutputBooster {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature=(
        n_boosters,
        objective,
        num_threads,
        monotone_constraints,
        force_children_to_bound_parent,
        missing,
        allow_missing_splits,
        create_missing_branch,
        terminate_missing_features,
        missing_node_treatment,
        log_iterations,
    ))]
    pub fn new(
        n_boosters: usize,
        objective: &str,
        num_threads: Option<usize>,
        monotone_constraints: HashMap<usize, i8>,
        force_children_to_bound_parent: bool,
        missing: f64,
        allow_missing_splits: bool,
        create_missing_branch: bool,
        terminate_missing_features: HashSet<usize>,
        missing_node_treatment: &str,
        log_iterations: usize,
    ) -> PyResult<Self> {
        let objective_ = to_value_error(serde_plain::from_str(objective))?;
        let missing_node_treatment_ = to_value_error(serde_plain::from_str(missing_node_treatment))?;
        let monotone_constraints_ = int_map_to_constraint_map(monotone_constraints)?;

        let booster = CrateMultiOutputBooster::default()
            .set_objective(objective_)
            .set_num_threads(num_threads)
            .set_monotone_constraints(Some(monotone_constraints_))
            .set_force_children_to_bound_parent(force_children_to_bound_parent)
            .set_missing(missing)
            .set_allow_missing_splits(allow_missing_splits)
            .set_create_missing_branch(create_missing_branch)
            .set_terminate_missing_features(terminate_missing_features)
            .set_missing_node_treatment(missing_node_treatment_)
            .set_log_iterations(log_iterations)
            .set_n_boosters(n_boosters);

        Ok(MultiOutputBooster { booster })
    }

    #[setter]
    fn set_n_boosters(&mut self, value: usize) -> PyResult<()> {
        self.booster = self.booster.clone().set_n_boosters(value);
        Ok(())
    }
    #[setter]
    fn set_objective(&mut self, value: &str) -> PyResult<()> {
        let objective_ = to_value_error(serde_plain::from_str(value))?;
        self.booster = self.booster.clone().set_objective(objective_);
        Ok(())
    }
    #[setter]
    fn set_num_threads(&mut self, value: Option<usize>) -> PyResult<()> {
        self.booster = self.booster.clone().set_num_threads(value);
        Ok(())
    }
    #[setter]
    fn set_monotone_constraints(&mut self, value: HashMap<usize, i8>) -> PyResult<()> {
        let map = int_map_to_constraint_map(value)?;
        self.booster = self.booster.clone().set_monotone_constraints(Some(map));
        Ok(())
    }
    #[setter]
    fn set_force_children_to_bound_parent(&mut self, value: bool) -> PyResult<()> {
        self.booster = self.booster.clone().set_force_children_to_bound_parent(value);
        Ok(())
    }
    #[setter]
    fn set_missing(&mut self, value: f64) -> PyResult<()> {
        self.booster = self.booster.clone().set_missing(value);
        Ok(())
    }
    #[setter]
    fn set_allow_missing_splits(&mut self, value: bool) -> PyResult<()> {
        self.booster = self.booster.clone().set_allow_missing_splits(value);
        Ok(())
    }
    #[setter]
    fn set_create_missing_branch(&mut self, value: bool) -> PyResult<()> {
        self.booster = self.booster.clone().set_create_missing_branch(value);
        Ok(())
    }
    #[setter]
    fn set_terminate_missing_features(&mut self, value: HashSet<usize>) -> PyResult<()> {
        self.booster = self.booster.clone().set_terminate_missing_features(value);
        Ok(())
    }
    #[setter]
    fn set_missing_node_treatment(&mut self, value: &str) -> PyResult<()> {
        let missing_node_treatment_ = to_value_error(serde_plain::from_str(value))?;
        self.booster = self.booster.clone().set_missing_node_treatment(missing_node_treatment_);
        Ok(())
    }
    #[setter]
    fn set_log_iterations(&mut self, value: usize) -> PyResult<()> {
        self.booster = self.booster.clone().set_log_iterations(value);
        Ok(())
    }

    #[getter]
    fn base_score<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<f64>>> {
        Ok(self
            .booster
            .boosters
            .iter()
            .map(|b| b.base_score)
            .collect::<Vec<_>>()
            .into_pyarray_bound(py))
    }

    #[getter]
    fn number_of_trees<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<usize>>> {
        Ok(self
            .booster
            .boosters
            .iter()
            .map(|b| b.get_prediction_trees().len())
            .collect::<Vec<_>>()
            .into_pyarray_bound(py))
    }

    pub fn fit(
        &mut self,
        flat_data: PyReadonlyArray1<f64>,
        rows: usize,
        cols: usize,
        y: PyReadonlyArray1<f64>,
        budget: f32,
        sample_weight: Option<PyReadonlyArray1<f64>>,
        alpha: Option<f32>,
        reset: Option<bool>,
        categorical_features: Option<HashSet<usize>>,
        timeout: Option<f32>,
    ) -> PyResult<()> {
        let flat_data = flat_data.as_slice()?;
        let data = Matrix::new(flat_data, rows, cols);

        let y = y.as_slice()?;
        let y_data = Matrix::new(y, rows, self.booster.n_boosters);

        let sample_weight_ = match sample_weight.as_ref() {
            Some(sw) => {
                let sw_slice = sw.as_slice()?;
                Some(sw_slice)
            }
            None => None,
        };

        match self.booster.fit(
            &data,
            &y_data,
            sample_weight_,
            alpha,
            budget,
            reset,
            categorical_features,
            timeout,
        ) {
            Ok(m) => Ok(m),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }?;

        Ok(())
    }

    pub fn predict<'py>(
        &self,
        py: Python<'py>,
        flat_data: PyReadonlyArray1<f64>,
        rows: usize,
        cols: usize,
        parallel: Option<bool>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let flat_data = flat_data.as_slice()?;
        let data = Matrix::new(flat_data, rows, cols);
        let parallel = parallel.unwrap_or(true);
        Ok(self.booster.predict(&data, parallel).into_pyarray_bound(py))
    }

    pub fn predict_proba<'py>(
        &self,
        py: Python<'py>,
        flat_data: PyReadonlyArray1<f64>,
        rows: usize,
        cols: usize,
        parallel: Option<bool>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let flat_data = flat_data.as_slice()?;
        let data = Matrix::new(flat_data, rows, cols);
        let parallel = parallel.unwrap_or(true);
        Ok(self.booster.predict_proba(&data, parallel).into_pyarray_bound(py))
    }

    pub fn save_booster(&self, path: &str) -> PyResult<()> {
        match self.booster.save_booster(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn json_dump(&self) -> PyResult<String> {
        match self.booster.json_dump() {
            Ok(m) => Ok(m),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn insert_metadata(&mut self, key: String, value: String) -> PyResult<()> {
        self.booster.insert_metadata(key, value);
        Ok(())
    }

    pub fn get_metadata(&self, key: String) -> PyResult<String> {
        match self.booster.get_metadata(&key) {
            Some(m) => Ok(m),
            None => Err(PyKeyError::new_err(format!(
                "No value associated with provided key {}",
                key
            ))),
        }
    }

    #[classmethod]
    pub fn load_booster(_: &Bound<'_, PyType>, path: String) -> PyResult<Self> {
        let booster = match CrateMultiOutputBooster::load_booster(path.as_str()) {
            Ok(m) => Ok(m),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }?;
        Ok(MultiOutputBooster { booster })
    }

    #[classmethod]
    pub fn from_json(_: &Bound<'_, PyType>, json_str: &str) -> PyResult<Self> {
        let booster = match CrateMultiOutputBooster::from_json(json_str) {
            Ok(m) => Ok(m),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }?;
        Ok(MultiOutputBooster { booster })
    }

    pub fn get_params(&self, py: Python) -> PyResult<PyObject> {
        let objective_ = to_value_error(serde_plain::to_string::<Objective>(&self.booster.objective))?;
        let missing_node_treatment_ = to_value_error(serde_plain::to_string::<MissingNodeTreatment>(
            &self.booster.missing_node_treatment,
        ))?;
        let monotone_constraints_: HashMap<usize, i8> = self
            .booster
            .monotone_constraints
            .as_ref()
            .unwrap_or(&HashMap::new())
            .iter()
            .map(|(f, c)| {
                let c_ = match c {
                    Constraint::Negative => -1,
                    Constraint::Positive => 1,
                    Constraint::Unconstrained => 0,
                };
                (*f, c_)
            })
            .collect();

        let key_vals: Vec<(&str, PyObject)> = vec![
            ("objective", objective_.to_object(py)),
            ("num_threads", self.booster.num_threads.to_object(py)),
            ("allow_missing_splits", self.booster.allow_missing_splits.to_object(py)),
            ("monotone_constraints", monotone_constraints_.to_object(py)),
            ("missing", self.booster.missing.to_object(py)),
            (
                "create_missing_branch",
                self.booster.create_missing_branch.to_object(py),
            ),
            (
                "terminate_missing_features",
                self.booster.terminate_missing_features.to_object(py),
            ),
            ("missing_node_treatment", missing_node_treatment_.to_object(py)),
            ("log_iterations", self.booster.log_iterations.to_object(py)),
            (
                "force_children_to_bound_parent",
                self.booster.force_children_to_bound_parent.to_object(py),
            ),
        ];
        let dict = key_vals.into_py_dict_bound(py);

        Ok(dict.to_object(py))
    }
}
