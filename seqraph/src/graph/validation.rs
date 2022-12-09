use crate::*;

#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidPatternRange(String, PatternLocation, usize, usize),
    InvalidPattern(PatternLocation),
}

impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub fn validate_pattern_indexing_range_at(
        &self,
        location: impl IntoPatternLocation,
        start: usize,
        end: usize,
    ) -> Result<(), ValidationError> {
        if end - start > 0 {
            self.validate_pattern_range_at(
                location,
                start,
                end,
            )
        } else {
            Err(ValidationError::InvalidPatternRange(
                "No more than a single index in range path".into(),
                location.into_pattern_location(),
                start,
                end,
            ))
        }
    }
    pub fn validate_pattern_range_at(
        &self,
        location: impl IntoPatternLocation,
        start: usize,
        end: usize,
    ) -> Result<(), ValidationError> {
        let location = location.into_pattern_location();
        let pattern = self.get_pattern_at(&location)
            .map_err(|_|
                ValidationError::InvalidPattern(
                    location,
                )
            )?;
        if end >= pattern.len() {
            Err(ValidationError::InvalidPatternRange(
                format!("End index {} out of pattern range {}", end, pattern.len()),
                location,
                start,
                end,
            ))
        } else if end < start {
            Err(ValidationError::InvalidPatternRange(
                format!("end < start: {} < {}", end, start),
                location,
                start,
                end,
            ))
        } else {
            Ok(())
        }
    }
}