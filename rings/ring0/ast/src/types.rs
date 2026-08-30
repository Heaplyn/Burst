use std::collections::HashMap;
use crate::{LayerKind, LayerId};

/// the actual types for layerscript rn
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// bit precise stuff like i32 or b8
    BitPrecise(char, u32),
    /// names for types we made ourselves
    Named(String),
    /// pointers for bare metal stuff
    Pointer(Box<Type>),
    /// arrays for holding a bunch of things
    Array(Box<Type>, usize),
    /// refinements for smt checks like x < 10
    Where(Box<Type>, Box<Expression>),
    /// unit type for when there is nothing
    Unit,
}

/// everything we can compute or do math with
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// just a variable name
    Variable(String),
    /// a whole number
    LiteralInt(i64),
    /// a number with a decimal
    LiteralFloat(f64),
    /// true or false bits
    LiteralBool(bool),
    /// text inside quotes
    LiteralString(String),
    /// bit precise type used as a value
    TypeLiteral { Kind: char, Bits: u32 },
    /// two things joined by an operator like + or <
    BinaryOp {
        Op: String,
        Lhs: Box<Expression>,
        Rhs: Box<Expression>,
    },
    /// one thing with an operator like *ptr
    UnaryOp {
        Op: String,
        Target: Box<Expression>,
    },
    /// calling a function with args
    FunctionCall {
        Name: String,
        Args: Vec<Expression>,
    },
    /// reaching into a struct like cpu.rax
    MemberAccess {
        Target: Box<Expression>,
        Member: String,
    },
    /// bit precise type specifically for the lexer
    BitPreciseType {
        Kind: char,
        Bits: u32,
    },
    /// indexing into an array or pointer like ptr[index]
    IndexAccess {
        Target: Box<Expression>,
        Index: Box<Expression>,
    }
}

/// function params with names and types
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub Name: String,
    pub Type_: Type,
}

/// struct fields with names and types
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub Name: String,
    pub Type_: Type,
}

/// enum variants with optional payloads
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub Name: String,
    pub Payload: Option<Type>,
}

/// generic params for type functions
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub Name: String,
    pub Bound: Option<Type>,
}

/// pattern matching variants
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// matches anything (_)
    Wildcard,
    /// matches a specific value
    Literal(Expression),
    /// matches and binds to a name
    Variable(String),
    /// matches an enum variant
    Variant(String, Option<Box<Pattern>>),
}

/// logic that runs when variables change
#[derive(Debug, Clone, PartialEq)]
pub struct VariableHook {
    pub Kind: HookKind,
    pub Body: Vec<LayerKind>,
    
}
impl VariableHook {
    pub fn New(Kind: HookKind, Body: Vec<LayerKind>) -> Self {
        Self { Kind, Body }
    }
}

/// the different kinds of variable behaviors
#[derive(Debug, Clone, PartialEq)]
pub enum HookKind {
    OnChange,
    OnRead,
    OnAssign,
    OnDrop,
    OnError,
}

/// where we store types for each layer
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TypeStorage {
    pub DefinedTypes: HashMap<String, TypeDefinition>,
    pub TypeAliases: HashMap<String, Type>,
}

/// full definition of a user type
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDefinition {
    pub Name: String,
    pub Kind: TypeKind,
    pub SourceLocation: SourceLocation,
    pub Docs: Option<String>,
}

/// the physical layout of a type
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Struct(Vec<StructField>),
    Enum(Vec<EnumVariant>),
    Alias(Box<Type>),
    Generic(Vec<GenericParam>),
}

/// tracking where code is in the files
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub File: String,
    pub Line: usize,
    pub Column: usize,
}

impl SourceLocation {
    /// for code that the compiler just knows
    pub fn Builtin() -> Self {
        Self {
            File: "<builtin>".to_string(),
            Line: 0,
            Column: 0,
        }
    }
}

/// values for metadata like doc comments
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

/// compiler directives like #[inline]
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompilerResult {
    Error(usize),//ErrorCode
    Success,
}

/// hints for the optimizer to go fast
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationHints {
    pub AggressiveLoopFolding: bool,
    pub TraceObservability: ObservabilityMode,
    pub RegisterPressure: RegisterPressureMode,
    pub InlineThreshold: Option<usize>,
}

/// how much the compiler should care about traces
#[derive(Debug, Clone, PartialEq)]
pub enum ObservabilityMode {
    Strict,
    Relaxed,
    Aggressive,
}

/// how many registers the compiler can use
#[derive(Debug, Clone, PartialEq)]
pub enum RegisterPressureMode {
    Low,
    High,
    Auto,
}

/// flags for the observability boundary
#[derive(Debug, Clone, PartialEq)]
pub struct ObservabilityFlags {
    pub ObservableValues: Vec<ObservableValue>,
    pub AffectsOutput: bool,
    pub AffectsHardware: bool,
    pub ObservableToTrace: bool,
}

/// values that we can actually see outside the program
#[derive(Debug, Clone, PartialEq)]
pub enum ObservableValue {
    Register(String),
    MemoryAddress(Expression),
    ReturnValue,
    SideEffect,
    Variable(String),
}

/// logical rules for smt and pomset
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// refined types like x > 0
    RefinedType {
        Variable: String,
        Condition: String,
    },
    /// making sure things are safe
    Safety {
        Condition: String,
        ErrorMessage: String,
    },
    /// partial ordering for parallel tasks
    POMSET {
        Before: LayerId,
        After: LayerId,
    },
}

/// info for the layertrace runtime
#[derive(Debug, Clone, PartialEq)]
pub struct TraceInfo {
    pub TraceId: String,
    pub Depth: usize,
    pub Context: TraceContext,
    pub TypeEnv: TypeStorage,
}

impl TraceInfo {
    pub fn default() -> Self {
        Self {
            TraceId: String::new(),
            Depth: 0,
            Context: TraceContext::Root,
            TypeEnv: TypeStorage::default(),
        }
    }
}

/// what the trace is actually inside of
#[derive(Debug, Clone, PartialEq)]
pub enum TraceContext {
    Root,
    Function { Name: String },
    Loop { Iteration: usize },
    Block,
}
