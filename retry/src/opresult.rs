pub enum OperationResult<T, E> {
    Ok(T),
    Retry(E),
    Err(E),
}

impl<T, E> From<Result<T, E>> for OperationResult<T, E> {
    fn from(item: Result<T, E>) -> Self {
        match item {
            Ok(v) => OperationResult::Ok(v),
            Err(e) => OperationResult::Retry(e),
        }
    }
}

impl<T, E> OperationResult<T, E> {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    pub fn is_retry(&self) -> bool {
        matches!(self, Self::Retry(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err(_))
    }
}
