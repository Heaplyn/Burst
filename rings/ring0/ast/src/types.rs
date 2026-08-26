use std::collections::HashMap;

// ============================================
// Core Primitive Types
// ============================================

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    BitPrecise(char, u32),  // e.g., i32, u16, b8, f64
    Named(String),          // User-defined types
    Pointer(Box<Type>),
    Array(Box<Type>, usize),
    Where(Box<Type>, Box<Expression>),
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Variable(String),
    LiteralInt(i64),
    LiteralFloat(f64),
    LiteralBool(bool),
    LiteralString(String),
    TypeLiteral { Kind: char, Bits: u32 },
    BinaryOp {
        Op: String,
        Lhs: Box<Expression>,
        Rhs: Box<Expression>,
    },
    UnaryOp {
        Op: String,
        Target: Box<Expression>,
    },
    FunctionCall {
        Name: String,
        Args: Vec<Expression>,
    },
    BitPreciseType {
        Kind: char,
        Bits: u32,
    }
}


// ============================================
// Structure Components
// ============================================

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub Name: String,
    pub Type_: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub Name: String,
    pub Type_: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub Name: String,
    pub Payload: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub Name: String,
    pub Bound: Option<Type>,
}

// ============================================
// Patterns & Hooks
// ============================================

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Literal(Expression),
    Variable(String),
    Variant(String, Option<Box<Pattern>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableHook {
    pub Kind: HookKind,
    pub Callback: String, // Function name or closure ID
}

#[derive(Debug, Clone, PartialEq)]
pub enum HookKind {
    OnChange,
    OnRead,
    OnAssign,
    OnDrop,
    OnError,
}

// ============================================
// Type Storage & Environment
// ============================================

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TypeStorage {
    pub DefinedTypes: HashMap<String, TypeDefinition>,
    pub TypeAliases: HashMap<String, Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDefinition {
    pub Name: String,
    pub Kind: TypeKind,
    pub SourceLocation: SourceLocation,
    pub Docs: Option<String>,
    pub Attributes: HashMap<String, MetadataValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Struct(Vec<StructField>),
    Enum(Vec<EnumVariant>),
    Alias(Box<Type>),
    Generic(Vec<GenericParam>),
}

// ============================================
// Metadata & Observability
// ============================================

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub File: String,
    pub Line: usize,
    pub Column: usize,
}

impl SourceLocation {
    pub fn Builtin() -> Self {
        Self {
            File: "<builtin>".to_string(),
            Line: 0,
            Column: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<MetadataValue>),
    Map(HashMap<String, MetadataValue>),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    Inline,
    NoInline,
    Align(usize),
    Section(String),
    Cold,
    Hot,
    Unsafe,
    Extern,
    HavocControl {
        CacheInvalidate: Vec<String>,
        CacheKeep: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationHints {
    pub AggressiveLoopFolding: bool,
    pub TraceObservability: ObservabilityMode,
    pub RegisterPressure: RegisterPressureMode,
    pub InlineThreshold: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObservabilityMode {
    Strict,
    Relaxed,
    Aggressive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegisterPressureMode {
    Low,
    High,
    Auto,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObservabilityFlags {
    pub ObservableValues: Vec<ObservableValue>,
    pub AffectsOutput: bool,
    pub AffectsHardware: bool,
    pub ObservableToTrace: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObservableValue {
    Register(String),
    MemoryAddress(Expression),
    ReturnValue,
    SideEffect,
    Variable(String),
}

// ============================================
// Constraints & Trace
// ============================================

#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    RefinedType {
        Variable: String,
        Condition: String,
    },
    Safety {
        Condition: String,
        ErrorMessage: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceInfo {
    pub TraceId: String,
    pub Depth: usize,
    pub Context: TraceContext,
    pub TypeEnv: TypeStorage,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraceContext {
    Root,
    Function { Name: String },
    Loop { Iteration: usize },
    Block,
}
