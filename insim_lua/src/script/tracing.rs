pub(crate) struct Tracing {}

impl mlua::UserData for Tracing {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("debug", |_, string: String| {
            tracing::debug!("{}", string);
            Ok(())
        });

        methods.add_function("info", |_, string: String| {
            tracing::info!("{}", string);
            Ok(())
        });

        methods.add_function("error", |_, string: String| {
            tracing::error!("{}", string);
            Ok(())
        });
    }
}
