use slint::{ModelRc, VecModel};

#[derive(Debug)]
pub enum SlintOption<T> {
    None,
    Some(T)
}

impl<T> From<Option<T>> for SlintOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(val) => Self::Some(val),
            None => Self::None,
        }
    }
}

impl<T, U> From<SlintOption<T>> for ModelRc<U>
where T: Clone + 'static, U: Clone + 'static + From<T>
{
    fn from(value: SlintOption<T>) -> Self {
        match value {
            SlintOption::None => ModelRc::default(),
            SlintOption::Some(val) => ModelRc::new(VecModel::from(vec![val.into()])),
        }
    }
}