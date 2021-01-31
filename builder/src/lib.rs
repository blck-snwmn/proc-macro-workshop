use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;

    let q = quote! {
        use std::error::Error;
        pub struct CommandBuilder {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }
        impl CommandBuilder {
            pub fn executable(&mut self, executable: String) -> &mut Self {
                self.executable = Some(executable);
                self
            }
            pub fn args(&mut self, args: Vec<String>) -> &mut Self {
                self.args = Some(args);
                self
            }
            pub fn env(&mut self, env: Vec<String>) -> &mut Self {
                self.env = Some(env);
                self
            }
            pub fn current_dir(&mut self, current_dir: String) -> &mut Self {
                self.current_dir = Some(current_dir);
                self
            }
            pub fn build(&mut self) -> Result<Command, Box<dyn Error>> {
                match (self.executable.take(), self.args.take(), self.env.take(), self.current_dir.take()){
                    (Some(ex),Some(a),Some(ev),Some(cd)) =>Ok(
                        Command{
                            executable: ex,
                            args: a,
                            env: ev,
                            current_dir: cd,
                        }
                    ),
                    _ =>  Err("a")?, // 手抜き
                }
            }
        }

        impl Command {
            pub fn builder() -> CommandBuilder {
                CommandBuilder {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }
    };
    q.into()
}
