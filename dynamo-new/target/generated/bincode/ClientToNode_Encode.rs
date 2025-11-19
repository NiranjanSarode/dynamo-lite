impl :: bincode :: Encode for ClientToNode
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        match self
        {
            Self ::ClientPut { key, value, metadata, client_addr, request_id }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (0u32), encoder) ?
                ; :: bincode :: Encode :: encode(key, encoder) ?; :: bincode
                :: Encode :: encode(value, encoder) ?; :: bincode :: Encode ::
                encode(metadata, encoder) ?; :: bincode :: Encode ::
                encode(client_addr, encoder) ?; :: bincode :: Encode ::
                encode(request_id, encoder) ?; core :: result :: Result ::
                Ok(())
            }, Self ::ClientGet { key, client_addr, request_id }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (1u32), encoder) ?
                ; :: bincode :: Encode :: encode(key, encoder) ?; :: bincode
                :: Encode :: encode(client_addr, encoder) ?; :: bincode ::
                Encode :: encode(request_id, encoder) ?; core :: result ::
                Result :: Ok(())
            },
        }
    }
}