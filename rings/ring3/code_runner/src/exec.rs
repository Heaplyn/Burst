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
                        // Third field (fallback) is ignored here because
                        // parameters don't have a value at declaration time —
                        // arguments are checked at the call site in `eval.rs`
                        // where the fallback is actually applied.
                        Type::Where(Type_, Expr, _) => {
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
                        }
                        other => {
                            self.Context.DeclareVariable(&Param.Name, other)?;
                        }
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

                // Refinement + `else` fallback:
                //   `val: u32 where val >= 10 && val <= 1000 else 0`
                // Bind the candidate value under `Name` (so the predicate can
                // reference it), evaluate the predicate. If it's false and we
                // have a fallback, evaluate the fallback and use *that* as the
                // stored value. If no fallback and the predicate fails, error.
                if let Some(Type::Where(_Base, Predicate, Fallback)) = TypeAnnotation {
                    self.Context.SetVariable(Name, Value.clone(), *IsMutable);
                    let PredResult = self.EvaluateExpression(Predicate)?;
                    if PredResult == Value::Bool(false) {
                        if let Some(FbExpr) = Fallback {
                            Value = self.EvaluateExpression(FbExpr)?;
                        } else {
                            return Err(CompilerError::TypeError(format!(
                                "Variable '{}' failed refinement check with no `else` fallback",
                                Name
                            )));
                        }
                    }
                }

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
            LayerKind::Block => {
                self.RunBlock(&Layer.Children)
            }

            LayerKind::Return { Value: Expr } => {
                let Val = if let Some(Expr) = Expr {
                    self.EvaluateExpression(Expr)?
                } else {
                    Value::Unit
                };
                self.ReturnValue = Some(Val);
                Ok(Value::Unit)
            }

            LayerKind::Conditional { Condition, HasElse } => {
                let ConditionVal = self.EvaluateExpression(Condition)?;
                
                if ConditionVal == Value::Bool(true) {
                    if let Some(ThenBranch) = Layer.Children.get(0) {
                        self.RunLayer(ThenBranch)?;
                    }
                } else if *HasElse {
                    if let Some(ElseBranch) = Layer.Children.get(1) {
                        self.RunLayer(ElseBranch)?;
                    }
                }
                Ok(Value::Unit)
            }
            LayerKind::Loop { Kind, .. } => {
                loop {
                    // 1. If it's a while loop, check the condition first
                    if let LoopKind::While(ConditionExpr) = Kind {
                        let ConditionVal = self.EvaluateExpression(ConditionExpr)?;
                        if ConditionVal == Value::Bool(false) {
                            break;
                        }
                    }
                    // 2. Execute the body block
                    let Result = self.RunBlock(&Layer.Children)?;
                    // 3. Propagate returns out of the loop
                    // (Any statement block that exits via a return is caught by looking at the last statement)
                    if let Some(LastChild) = Layer.Children.last() {
                        if let LayerKind::Return { .. } = &LastChild.Kind {
                            return Ok(Result); // Propagate the returned value immediately up the stack
                        }
                    }
                }
                Ok(Value::Unit)
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
            if self.ReturnValue.is_some() {
                break;
            }
            Result = self.RunLayer(Child)?;
        }

        Ok(Result)
    }
}

