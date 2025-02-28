use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

/// `This` implements the Opcode Operation for `Opcode::This`
///
/// Operation:
///  - Pushes `this` value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct This;

impl Operation for This {
    const NAME: &'static str = "This";
    const INSTRUCTION: &'static str = "INST - This";
    const COST: u8 = 1;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let this = context.vm.environments.get_this_binding()?;
        context.vm.push(this);
        Ok(CompletionType::Normal)
    }
}

/// `Super` implements the Opcode Operation for `Opcode::Super`
///
/// Operation:
///  - Pushes the current `super` value to the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Super;

impl Operation for Super {
    const NAME: &'static str = "Super";
    const INSTRUCTION: &'static str = "INST - Super";
    const COST: u8 = 3;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let home_object = {
            let env = context
                .vm
                .environments
                .get_this_environment()
                .as_function()
                .expect("super access must be in a function environment");
            let this = env
                .get_this_binding()?
                .expect("`get_this_environment` ensures this returns `Some`");
            let function_object = env.slots().function_object().borrow();
            let function = function_object
                .as_function()
                .expect("must be function object");
            function.get_home_object().or(this.as_object()).cloned()
        };

        let value = home_object
            .map(|o| o.__get_prototype_of__(context))
            .transpose()?
            .flatten()
            .map_or_else(JsValue::null, JsValue::from);

        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

/// `SuperCallPrepare` implements the Opcode Operation for `Opcode::SuperCallPrepare`
///
/// Operation:
///  - Get the super constructor and the new target of the current environment.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SuperCallPrepare;

impl Operation for SuperCallPrepare {
    const NAME: &'static str = "SuperCallPrepare";
    const INSTRUCTION: &'static str = "INST - SuperCallPrepare";
    const COST: u8 = 3;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let this_env = context
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");
        let active_function = this_env.slots().function_object().clone();
        let super_constructor = active_function
            .__get_prototype_of__(context)
            .expect("function object must have prototype");

        context
            .vm
            .push(super_constructor.map_or_else(JsValue::null, JsValue::from));
        Ok(CompletionType::Normal)
    }
}

/// `SuperCall` implements the Opcode Operation for `Opcode::SuperCall`
///
/// Operation:
///  - Execute the `super()` method.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SuperCall;

impl SuperCall {
    fn operation(context: &mut Context<'_>, argument_count: usize) -> JsResult<CompletionType> {
        let super_constructor_index = context.vm.stack.len() - argument_count - 1;
        let super_constructor = context.vm.stack[super_constructor_index].clone();
        let Some(super_constructor) = super_constructor.as_constructor() else {
            return Err(JsNativeError::typ()
                .with_message("super constructor object must be constructor")
                .into());
        };

        let this_env = context
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");

        let new_target = this_env
            .slots()
            .new_target()
            .expect("must have new.target")
            .clone();

        context.vm.push(new_target);

        super_constructor
            .__construct__(argument_count)
            .resolve(context)?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for SuperCall {
    const NAME: &'static str = "SuperCall";
    const INSTRUCTION: &'static str = "INST - SuperCall";
    const COST: u8 = 3;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value_count = context.vm.read::<u8>() as usize;
        Self::operation(context, value_count)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value_count = context.vm.read::<u16>() as usize;
        Self::operation(context, value_count)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value_count = context.vm.read::<u32>() as usize;
        Self::operation(context, value_count)
    }
}

/// `SuperCallSpread` implements the Opcode Operation for `Opcode::SuperCallSpread`
///
/// Operation:
///  - Execute the `super()` method where the arguments contain spreads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SuperCallSpread;

impl Operation for SuperCallSpread {
    const NAME: &'static str = "SuperCallSpread";
    const INSTRUCTION: &'static str = "INST - SuperCallSpread";
    const COST: u8 = 3;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        // Get the arguments that are stored as an array object on the stack.
        let arguments_array = context.vm.pop();
        let arguments_array_object = arguments_array
            .as_object()
            .expect("arguments array in call spread function must be an object");
        let arguments = arguments_array_object
            .borrow()
            .properties()
            .dense_indexed_properties()
            .expect("arguments array in call spread function must be dense")
            .clone();

        let super_constructor = context.vm.pop();

        let Some(super_constructor) = super_constructor.as_constructor() else {
            return Err(JsNativeError::typ()
                .with_message("super constructor object must be constructor")
                .into());
        };

        context.vm.push(super_constructor.clone());

        context.vm.push_values(&arguments);

        let this_env = context
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");

        let new_target = this_env
            .slots()
            .new_target()
            .expect("must have new.target")
            .clone();

        context.vm.push(new_target);

        super_constructor
            .__construct__(arguments.len())
            .resolve(context)?;
        Ok(CompletionType::Normal)
    }
}

/// `SuperCallDerived` implements the Opcode Operation for `Opcode::SuperCallDerived`
///
/// Operation:
///  - Execute the `super()` method when no constructor of the class is defined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SuperCallDerived;

impl Operation for SuperCallDerived {
    const NAME: &'static str = "SuperCallDerived";
    const INSTRUCTION: &'static str = "INST - SuperCallDerived";
    const COST: u8 = 3;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let argument_count = context.vm.frame().argument_count as usize;

        let this_env = context
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");
        let new_target = this_env
            .slots()
            .new_target()
            .expect("must have new target")
            .clone();
        let active_function = this_env.slots().function_object().clone();
        let super_constructor = active_function
            .__get_prototype_of__(context)
            .expect("function object must have prototype")
            .expect("function object must have prototype");

        if !super_constructor.is_constructor() {
            return Err(JsNativeError::typ()
                .with_message("super constructor object must be constructor")
                .into());
        }

        let arguments_start_index = context.vm.stack.len() - argument_count;
        context
            .vm
            .stack
            .insert(arguments_start_index, super_constructor.clone().into());

        context.vm.push(new_target);

        super_constructor
            .__construct__(argument_count)
            .resolve(context)?;
        Ok(CompletionType::Normal)
    }
}

/// `BindThisValue` implements the Opcode Operation for `Opcode::BindThisValue`
///
/// Operation:
///  - Binds `this` value and initializes the instance elements.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BindThisValue;

impl Operation for BindThisValue {
    const NAME: &'static str = "BindThisValue";
    const INSTRUCTION: &'static str = "INST - BindThisValue";
    const COST: u8 = 6;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        // Taken from `SuperCall : super Arguments` steps 7-12.
        //
        // <https://tc39.es/ecma262/#sec-super-keyword-runtime-semantics-evaluation>

        let result = context
            .vm
            .pop()
            .as_object()
            .expect("construct result should be an object")
            .clone();

        // 7. Let thisER be GetThisEnvironment().
        let this_env = context
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");

        // 8. Perform ? thisER.BindThisValue(result).
        this_env.bind_this_value(result.clone())?;

        // 9. Let F be thisER.[[FunctionObject]].
        // SKIP: 10. Assert: F is an ECMAScript function object.
        let active_function = this_env.slots().function_object().clone();

        // 11. Perform ? InitializeInstanceElements(result, F).
        result.initialize_instance_elements(&active_function, context)?;

        // 12. Return result.
        context.vm.push(result);
        Ok(CompletionType::Normal)
    }
}
