#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::sync::atomic::{AtomicUsize, Ordering};

pub mod types;
pub use types::*;


use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}
/// the main building block for all code in burst
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    /// unique id for this layer
    pub Id: LayerId,
    /// what kind of thing this is (function block variable etc)
    pub Kind: LayerKind,
    /// extra info like docs and hints
    pub Metadata: LayerMetadata,
    /// nested layers inside this one
    pub Children: Vec<Layer>,
    /// logical constraints for proof erasure
    pub Constraints: Vec<Constraint>,
    /// boundary flags for observability
    pub Observability: ObservabilityFlags,
    /// types defined specifically for this scope
    pub TypeStorage: TypeStorage,
    /// runtime info for layertrace
    pub TraceInfo: TraceInfo,
}

/// just a wrapper for layer string ids
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct LayerId(usize );

/// all the different things a layer can represent
#[derive(Debug, Clone, PartialEq)]
pub enum LayerKind {
    /// the whole script
    Program,
    /// a logic procedure with params
    Function {
        Name: String,
        Params: Vec<Param>,
        ReturnType: Option<Type>,
        IsUnsafe: bool,
        IsExtern: bool,
    },
    /// binding a value to a name
    VariableBinding {
        Name: String,
        TypeAnnotation: Option<Type>,
        IsMutable: bool,
        Hooks: Vec<VariableHook>,
        InitialValue: Option<Expression>,
    },
    /// updating an existing variable
    Assignment {
        Target: Expression,
        Value: Expression,
    },
    /// a single standalone expression
    Expression(Expression),
    /// a curly brace scope
    Block,
    /// iterative control flow
    Loop {
        Label: Option<String>,
        Kind: LoopKind,
    },
    /// branching logic
    Conditional {
        Condition: Expression,
        HasElse: bool,
    },
    /// one arm of a match block
    MatchArm {
        Pattern: Pattern,
        Guard: Option<Expression>,
    },
    /// stopping everything with a crash
    Panic,
    /// telling the compiler this path is impossible
    Unreachable,
    /// invalidating register caches
    Havoc {
        Target: Expression,
    },
    /// jumping to hardware boundary
    Interrupt {
        Syscall: String,
    },
    /// physical layout definition
    Struct {
        Name: String,
        Fields: Vec<StructField>,
        IsPacked: bool,
    },
    /// algebraic data type
    Enum {
        Name: String,
        Variants: Vec<EnumVariant>,
    },
}

/// the specific flavor of the loop
#[derive(Debug, Clone, PartialEq)]
pub enum LoopKind {
    /// loop forever
    Infinite,
    /// loop while true
    While(Expression),
    /// loop with init and update
    For {
        Init: Box<Layer>,
        Condition: Expression,
        Update: Box<Layer>,
    },
}

/// all the secondary info for a layer
#[derive(Debug, Clone, PartialEq)]
pub struct LayerMetadata {
    pub SourceLocation: SourceLocation,
    pub Docs: Option<String>,
    pub Directives: Vec<Directive>,
    pub Optimization: OptimizationHints,
    pub Custom: HashMap<String, MetadataValue>,
}

/// helper to make creating layers easier
pub struct LayerBuilder {
    layer: Layer,
}
pub static LayerAddress: AtomicUsize = AtomicUsize::new(0);
impl LayerBuilder {
    /// starts a new layer builder with a timestamped id
    pub fn New(kind: LayerKind, source_location: SourceLocation) -> Self {
        Self {
            layer: Layer {
                Id: LayerId( LayerAddress.fetch_add(1, Ordering::SeqCst)),
                Kind: kind,
                Metadata: LayerMetadata {
                    SourceLocation: source_location,
                    Docs: None,
                    Directives: Vec::new(),
                    Optimization: OptimizationHints {
                        AggressiveLoopFolding: false,
                        TraceObservability: ObservabilityMode::Strict,
                        RegisterPressure: RegisterPressureMode::Auto,
                        InlineThreshold: None,
                    },
                    Custom: HashMap::new(),
                },
                Children: Vec::new(),
                Constraints: Vec::new(),
                Observability: ObservabilityFlags {
                    ObservableValues: Vec::new(),
                    AffectsOutput: false,
                    AffectsHardware: false,
                    ObservableToTrace: true,
                },
                TypeStorage: TypeStorage::default(),
                TraceInfo: TraceInfo {
                    TraceId: "unknown".to_string(),
                    Depth: 0,
                    Context: TraceContext::Root,
                    TypeEnv: TypeStorage::default(),
                },
            },
        }
    }

    /// adds a doc comment to the layer
    pub fn WithDoc(mut self, doc: String) -> Self {
        self.layer.Metadata.Docs = Some(doc);
        self
    }

    /// nests a child layer inside
    pub fn WithChild(mut self, child: Layer) -> Self {
        self.layer.Children.push(child);
        self
    }

    /// nests a bunch of children at once
    pub fn WithChildren(mut self, children: Vec<Layer>) -> Self {
        self.layer.Children.extend(children);
        self
    }

    /// adds an smt or pomset rule
    pub fn WithConstraint(mut self, constraint: Constraint) -> Self {
        self.layer.Constraints.push(constraint);
        self
    }

    /// finishes building the layer
    pub fn Build(self) -> Layer {
        self.layer
    }
}

impl Layer {
    /// registers a new type in this scope
    pub fn AddType(&mut self, type_def: TypeDefinition) {
        self.TypeStorage.DefinedTypes.insert(type_def.Name.clone(), type_def);
    }
    
    /// checks if we are at the top level
    pub fn IsRoot(&self) -> bool {
        matches!(self.Kind, LayerKind::Program)
    }

    /// adds a pomset rule between two layers
    pub fn AddDependency(&mut self, before: LayerId, after: LayerId) {
        self.Constraints.push(Constraint::POMSET { Before: before, After: after });
    }
}
