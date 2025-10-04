#![deny(clippy::disallowed_methods)]
#![feature(test)]
#![feature(assert_matches)]
#![feature(try_blocks)]
//#![feature(hash_drain_filter)]
#![feature(slice_pattern)]
//#![feature(pin_macro)]
#![feature(exact_size_is_empty)]
#![feature(associated_type_defaults)]
//#![feature(return_position_impl_trait_in_trait)]
#![feature(type_changing_struct_update)]

pub mod direction;
pub mod path;

pub mod graph;
pub mod trace;

#[cfg(any(test, feature = "test-api"))]
pub mod tests;

#[cfg(not(any(test, feature = "test-api")))]
pub use std::collections::{
    HashMap,
    HashSet,
};
#[cfg(any(test, feature = "test-api"))]
pub use {
    ::charify,
    std::hash::{
        BuildHasherDefault,
        DefaultHasher,
    },
};
#[cfg(any(test, feature = "test-api"))]
pub type HashSet<T> =
    std::collections::HashSet<T, BuildHasherDefault<DefaultHasher>>;
#[cfg(any(test, feature = "test-api"))]
pub type HashMap<K, V> =
    std::collections::HashMap<K, V, BuildHasherDefault<DefaultHasher>>;

#[cfg(any(test, feature = "test-api"))]
pub use tests::{
    assert_parents,
    env::{
        Env1,
        TestEnv,
    },
    init_tracing,
};

// Auto-generated pub use statements
pub use crate::{
    direction::{
        Direction,
        Left,
        Right,
    },
    graph::{
        Hypergraph,
        HypergraphRef,
        getters::{
            ErrorReason,
            IndexWithPath,
            vertex::VertexSet,
        },
        kind::{
            BaseGraphKind,
            TokenOf,
        },
        vertex::{
            ChildPatterns,
            VertexIndex,
            child::{
                Child,
                ChildWidth,
            },
            data::VertexData,
            has_vertex_data::{
                HasVertexData,
                HasVertexDataMut,
            },
            has_vertex_index::{
                HasVertexIndex,
                ToChild,
            },
            location::{
                SubLocation,
                child::ChildLocation,
                pattern::IntoPatternLocation,
            },
            parent::{
                Parent,
                PatternIndex,
            },
            pattern::{
                IntoPattern,
                Pattern,
                id::PatternId,
                pattern_range::PatternRangeIndex,
                pattern_width,
            },
            token::{
                AsToken,
                NewTokenIndex,
                NewTokenIndices,
                tokenizing_iter,
            },
            wide::Wide,
        },
    },
    path::{
        RolePathUtils,
        accessors::{
            child::{
                PathChild,
                RootChildIndex,
                root::{
                    GraphRootChild,
                    RootChild,
                },
            },
            complete::PathComplete,
            has_path::{
                HasPath,
                HasRootedRolePath,
            },
            role::{
                End,
                PathRole,
                Start,
            },
            root::{
                GraphRoot,
                RootPattern,
            },
        },
        mutators::{
            adapters::IntoAdvanced,
            append::PathAppend,
            lower::PathLower,
            move_path::{
                advance::{
                    Advance,
                    CanAdvance,
                },
                key::{
                    MoveKey,
                    TokenPosition,
                },
                path::MovePath,
                retract::Retract,
                root::MoveRootIndex,
            },
            pop::PathPop,
            raise::PathRaise,
            simplify::PathSimplify,
        },
        structs::{
            query_range_path::FoldablePath,
            role_path::RolePath,
            rooted::{
                index_range::IndexRangePath,
                pattern_range::{
                    PatternPostfixPath,
                    PatternRangePath,
                },
                role_path::{
                    IndexEndPath,
                    IndexStartPath,
                    PatternEndPath,
                    RootedRolePath,
                },
                root::IndexRoot,
                split_path::RootedSplitPathRef,
            },
            sub_path::SubPath,
        },
    },
    trace::{
        StateDirection,
        TraceCtx,
        cache::{
            TraceCache,
            key::{
                directed::{
                    DirectedKey,
                    DirectedPosition,
                    down::DownKey,
                    up::UpKey,
                },
                props::{
                    CursorPosition,
                    LeafKey,
                    RootKey,
                    TargetKey,
                },
            },
            position::{
                Offset,
                PositionCache,
                SubSplitLocation,
            },
            vertex::{
                VertexCache,
                positions::DirectedPositions,
            },
        },
        child::{
            ChildTracePos,
            TraceBack,
            TraceSide,
            bands::{
                HasChildRoleIters,
                PostfixIterator,
            },
            iterator::{
                ChildIterator,
                ChildQueue,
            },
            state::{
                ChildState,
                PrefixStates,
            },
        },
        command::{
            PostfixCommand,
            PrefixCommand,
            RangeCommand,
        },
        has_graph::{
            HasGraph,
            HasGraphMut,
            TravKind,
        },
        node::{
            AsNodeTraceCtx,
            NodeTraceCtx,
        },
        pattern::{
            GetPatternCtx,
            GetPatternTraceCtx,
            HasPatternTraceCtx,
            PatternTraceCtx,
        },
        state::{
            BaseState,
            InnerKind,
            parent::{
                ParentBatch,
                ParentState,
            },
        },
        traceable::Traceable,
    },
};
