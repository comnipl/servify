

#[cfg(test)]
mod tests {
    mod SomeStruct {
        pub struct Server {
            pub a: String,
        }      
    }

    mod SomeStruct_AddHello {
        use super::SomeStruct;

        struct Request {
            n: usize
        }
        type Response = String;

        impl SomeStruct::Server {
            pub async fn add_hello(&mut self, req: Request) -> Response {
                self.__internal_add_hello(req.n)
            }

            async fn __internal_add_hello(&mut self, n: usize) -> Response {
                self.a.push_str(&"Hello".repeat(n));
                self.a.clone()
            }
            
        }

        trait ClientTrait {
            
        }

    }
}
