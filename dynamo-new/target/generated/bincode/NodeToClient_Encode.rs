impl :: bincode :: Encode for NodeToClient
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        match self
        {
            Self ::ClientPutRsp { key, request_id, client_addr }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (0u32), encoder) ?
                ; :: bincode :: Encode :: encode(key, encoder) ?; :: bincode
                :: Encode :: encode(request_id, encoder) ?; :: bincode ::
                Encode :: encode(client_addr, encoder) ?; core :: result ::
                Result :: Ok(())
            }, Self ::ClientGetRsp
            { key, request_id, values, metadata, client_addr }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (1u32), encoder) ?
                ; :: bincode :: Encode :: encode(key, encoder) ?; :: bincode
                :: Encode :: encode(request_id, encoder) ?; :: bincode ::
                Encode :: encode(values, encoder) ?; :: bincode :: Encode ::
                encode(metadata, encoder) ?; :: bincode :: Encode ::
                encode(client_addr, encoder) ?; core :: result :: Result ::
                Ok(())
            },
        }
    }
}