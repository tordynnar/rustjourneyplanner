#[derive(Debug, Clone)]
pub enum ErrorCategory {
    Loading, Input, Routing, Critical
}

#[derive(Debug, Clone)]
pub struct ErrorStatus {
    pub category : ErrorCategory,
    pub description : String
}

pub fn loadingerror<S: Into<String>>(description : S) -> ErrorStatus {
    ErrorStatus { category : ErrorCategory::Loading, description : description.into() }
}

pub fn inputerror<S: Into<String>>(description : S) -> ErrorStatus {
    ErrorStatus { category : ErrorCategory::Input, description : description.into() }
}

pub fn routingerror<S: Into<String>>(description : S) -> ErrorStatus {
    ErrorStatus { category : ErrorCategory::Routing, description : description.into() }
}

pub fn criticalerror<S: Into<String>>(description : S) -> ErrorStatus {
    ErrorStatus { category : ErrorCategory::Critical, description : description.into() }
}
