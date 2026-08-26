#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod types;
pub use types::*;

use std::collections::HashMap;

// ============================================
// Core Layer Structure
// ============================================

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub Id: LayerId,
    pub Kind: LayerKind,
    pub Metadata: LayerMetadata,
    pub Children: Vec<Layer>,
    pub Constraints: Vec<Constraint>,
    pub Observability: ObservabilityFlags,
    pub TypeStorage: TypeStorage,
    pub TraceInfo: TraceInfo,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct LayerId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum LayerKind {
    Program,
    Function {
        Name: String,
        Params: Vec<Param>,
        ReturnType: Option<Type>,
        IsUnsafe: bool,
        IsExtern: bool,
    },
    VariableBinding {
        Name: String,
        TypeAnnotation: Option<Type>,
        IsMutable: bool,
        Hooks: Vec<VariableHook>,
        InitialValue: Option<Expression>,
    },
    Assignment {
        Target: Expression,
        Value: Expression,
    },
    Expression(Expression),
    Block,
    Loop {
        Label: Option<String>,
    },
    Conditional {
        Condition: Expression,
        HasElse: bool,
    },
    MatchArm {
        Pattern: Pattern,
        Guard: Option<Expression>,
    },
    Panic,
    Unreachable,
    Havoc {
        Target: Expression,
    },
    Interrupt {
        Syscall: String,
    },
    Struct {
        Name: String,
        Fields: Vec<StructField>,
        IsPacked: bool,
    },
    Enum {
        Name: String,
        Variants: Vec<EnumVariant>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerMetadata {
    pub SourceLocation: SourceLocation,
    pub Docs: Option<String>,
    pub Directives: Vec<Directive>,
    pub Optimization: OptimizationHints,
    pub Custom: HashMap<String, MetadataValue>,
}

// ============================================
// Layer Builder
// ============================================

pub struct LayerBuilder {
    layer: Layer,
}

impl LayerBuilder {
    pub fn New(kind: LayerKind, source_location: SourceLocation) -> Self {
        Self {
            layer: Layer {
                Id: LayerId(format!("layer_{}", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_micros())),
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

    pub fn WithDoc(mut self, doc: String) -> Self {
        self.layer.Metadata.Docs = Some(doc);
        self
    }

    pub fn WithChild(mut self, child: Layer) -> Self {
        self.layer.Children.push(child);
        self
    }

    pub fn WithChildren(mut self, children: Vec<Layer>) -> Self {
        self.layer.Children.extend(children);
        self
    }

    pub fn Build(self) -> Layer {
        self.layer
    }
}

// ============================================
// Layer Logic
// ============================================

impl Layer {
    pub fn AddType(&mut self, type_def: TypeDefinition) {
        self.TypeStorage.DefinedTypes.insert(type_def.Name.clone(), type_def);
    }
    
    pub fn IsRoot(&self) -> bool {
        matches!(self.Kind, LayerKind::Program)
    }
}
