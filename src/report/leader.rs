use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use crate::genealogy::HarnessId;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Leader {
    BestHarness(HarnessId),
    HarnessAheadSoFar(HarnessId),
}

impl Leader {
    pub(super) fn id(self) -> HarnessId {
        match self {
            Self::BestHarness(id) | Self::HarnessAheadSoFar(id) => id,
        }
    }

    pub(super) fn named(self) -> &'static str {
        match self {
            Self::BestHarness(_) => "the best harness",
            Self::HarnessAheadSoFar(_) => "the harness that is ahead so far",
        }
    }
}

impl Display for Leader {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::BestHarness(id) => write!(formatter, "best harness {id}"),
            Self::HarnessAheadSoFar(id) => write!(formatter, "harness ahead so far {id}"),
        }
    }
}
