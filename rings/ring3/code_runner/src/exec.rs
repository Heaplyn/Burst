//! Layer execution: running statements/blocks and producing their [`Value`].

use ast::*;

use parser::error::{CompilerError, CompilerResult};
use crate::runner::CodeRunner;
use crate::value::Value;

impl CodeRunner {
    pub fn RunLayer(&mut self, Layer: &Layer) -> CompilerResult<Value> {
        match &Layer.Kind {
            LayerKind::Program => {
                for Child in &Layer.Children {
                    self.RunLayer(Child)?;
                }
                Ok(Value::Unit)
            }

            LayerKind::Function { Name, Params, ReturnType, .. } => {
                // Push function context
                self.Context.PushFrame(Name);
                for Param in Params {
                    match &Param.Type_ {
                        Type::Where(Type_,Expr ) => {
                            let Return = self.EvaluateExpression(Expr)?;
                            if !self.CheckType(&Return, Type_) {
                                return Err(CompilerError::TypeError(format!(
                                    "Parameter '{}' type mismatch",
                                    Param.Name
                                )));
                            }
                            if Return == Value::Bool(false) {
                                println!("Avoided invalid typing func");
                                return Err(CompilerError::TypeError(format!(
                                    "Parameter '{}' failed refinement check",
                                    Param.Name
                                )));
                            }
                            self.Context.DeclareVariable(&Param.Name, Type_)?;
                        
                        },
                        _ => return Err(CompilerError::TypeError(format!(
                            "Parameter '{}' missing type annotation",
                            Param.Name
                        ))),
                    }
                    }
                    
            

                // Execute function body
                let Result = self.RunBlock(&Layer.Children)?;

                // Pop context
                self.Context.PopFrame();

                // Check return type
                if let Some(ReturnType) = ReturnType {
                    if !self.CheckType(&Result, ReturnType) {
                        return Err(CompilerError::TypeError(format!(
                            "Return type mismatch in function '{}'",
                            Name
                        )));
                    }
                }
            
                Ok(Result)
            
        }
    
            
            // NOTE: LayerKind::FunctionCall is never emitted by the parser — calls arrive
            // as Expression::FunctionCall and are handled in EvaluateExpression.

            // A bare expression statement (e.g. `add(3, 4);` or `2 + 5 * 3;`) evaluates
            // to its value, so the top level runs as a sequence of expressions.
            LayerKind::Expression(Expr) => self.EvaluateExpression(Expr),

            LayerKind::VariableBinding { Name, TypeAnnotation, IsMutable, Hooks, InitialValue } => {
                // Evaluate initial value
                let mut Value = if let Some(Expr) = InitialValue {
                    self.EvaluateExpression(Expr)?
                } else {
                    // Use default value based on type
                    self.DefaultValue(TypeAnnotation)?
                };

                // Run on_assign hooks
                for Hook in Hooks {
                    if Hook.Kind != HookKind::OnAssign {
                        continue;
                    }
                    for child_kind in &Hook.Body {
                        if let LayerKind::Function { Name, Params, ReturnType, .. } = child_kind {
                            // Create a temporary Layer to run the function
                            let temp_func_layer = Layer {
                                Id: LayerId { Id: 0 }, // Placeholder ID
                                Kind: child_kind.clone(),
                                Metadata: LayerMetadata {
                                    SourceLocation: SourceLocation::Builtin(),
                                    Docs: None,
                                    Directives: Vec::new(),
                                    Optimization: OptimizationHints {
                                        AggressiveLoopFolding: false,
                                        TraceObservability: ObservabilityMode::Strict,
                                        RegisterPressure: RegisterPressureMode::Auto,
                                        InlineThreshold: None,
                                    },
                                    Custom: std::collections::HashMap::new(),
                                },
                                Children: vec![], // The body statements
                                Constraints: vec![],
                                Observability: ObservabilityFlags {
                                    ObservableValues: Vec::new(),
                                    AffectsOutput: true,
                                    AffectsHardware: true,
                                    ObservableToTrace: true,
                                },
                                TypeStorage: TypeStorage::default(),
                                VariableStorage: VariableStorage::default(),
                                TraceInfo: TraceInfo {
                                    TraceId: "unknown".to_string(),
                                    Depth: 0,
                                    Context: ast::TraceContext::Root,
                                    TypeEnv: TypeStorage::default(),
                                },
                            };
                            Value = self.RunLayer(&temp_func_layer)?;
                        }
                    }
                }

                // Store variable
                self.Context.SetVariable(Name, Value.clone(), *IsMutable);

                // Run on_change hooks
                for Hook in Hooks {
                    if Hook.Kind != HookKind::OnChange {
                        continue;
                    }
                    for child_kind in &Hook.Body {
                        if let LayerKind::Function { Name, Params, ReturnType, .. } = child_kind {
                            // Create a temporary Layer to run the function
                            let temp_func_layer = Layer {
                                Id: LayerId { Id: 0 }, // Placeholder ID
                                Kind: child_kind.clone(),
                                Metadata: LayerMetadata {
                                    SourceLocation: SourceLocation::Builtin(),
                                    Docs: None,
                                    Directives: Vec::new(),
                                    Optimization: OptimizationHints {
                                        AggressiveLoopFolding: false,
                                        TraceObservability: ObservabilityMode::Strict,
                                        RegisterPressure: RegisterPressureMode::Auto,
                                        InlineThreshold: None,
                                    },
                                    Custom: std::collections::HashMap::new(),
                                },
                                Children: vec![], // The body statements
                                Constraints: vec![],
                                Observability: ObservabilityFlags {
                                    ObservableValues: Vec::new(),
                                    AffectsOutput: true,
                                    AffectsHardware: true,
                                    ObservableToTrace: true,
                                },
                                TypeStorage: TypeStorage::default(),
                                VariableStorage: VariableStorage::default(),
                                TraceInfo: TraceInfo {
                                    TraceId: "unknown".to_string(),
                                    Depth: 0,
                                    Context: ast::TraceContext::Root,
                                    TypeEnv: TypeStorage::default(),
                                },
                            };
                            self.RunLayer(&temp_func_layer)?;
                        }
                    }
                }

                Ok(Value::Unit)
            }

            /*
            LayerKind::Havoc { Target, KeepCache } => {
                // Invalidate cache for target
                self.Context.Invalidate(Target);

                // Keep specified values in cache
                for Keep in KeepCache {
                    self.Context.KeepCache(Keep);
                }

                // Record observability
                self.Trace.RecordEvent(TraceEvent::Havoc {
                    Target: Target.to_string(),
                });

                Ok(Value::Unit)
            }


             */
            LayerKind::Return { Value: Expr } => {
                if let Some(Expr) = Expr {
                    let Val = self.EvaluateExpression(Expr)?;
                    Ok(Val)
                } else {
                    Ok(Value::Unit)
                }
            }

            _ => Err(CompilerError::RuntimeError(format!(
                "Unsupported layer kind: {:?}",
                Layer.Kind
            ))),
            
        }
    }

    pub(crate) fn RunBlock(&mut self, Children: &[Layer]) -> CompilerResult<Value> {
        let mut Result = Value::Unit;

        for Child in Children {
            Result = self.RunLayer(Child)?;

            // Check if we hit a return
            if let LayerKind::Return { .. } = &Child.Kind {
                break;
            }
        }

        Ok(Result)
    }
}

