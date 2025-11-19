impl :: bincode :: Encode for NodeToNode
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        match self
        {
            Self ::ForwardClientPut
            { coordinator, key, value, metadata, client_addr, request_id }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (0u32), encoder) ?
                ; :: bincode :: Encode :: encode(coordinator, encoder) ?; ::
                bincode :: Encode :: encode(key, encoder) ?; :: bincode ::
                Encode :: encode(value, encoder) ?; :: bincode :: Encode ::
                encode(metadata, encoder) ?; :: bincode :: Encode ::
                encode(client_addr, encoder) ?; :: bincode :: Encode ::
                encode(request_id, encoder) ?; core :: result :: Result ::
                Ok(())
            }, Self ::ForwardClientGet
            { coordinator, key, client_addr, request_id }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (1u32), encoder) ?
                ; :: bincode :: Encode :: encode(coordinator, encoder) ?; ::
                bincode :: Encode :: encode(key, encoder) ?; :: bincode ::
                Encode :: encode(client_addr, encoder) ?; :: bincode :: Encode
                :: encode(request_id, encoder) ?; core :: result :: Result ::
                Ok(())
            }, Self ::PutReq { from, to, key, value, clock, msg_id, handoff }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (2u32), encoder) ?
                ; :: bincode :: Encode :: encode(from, encoder) ?; :: bincode
                :: Encode :: encode(to, encoder) ?; :: bincode :: Encode ::
                encode(key, encoder) ?; :: bincode :: Encode ::
                encode(value, encoder) ?; :: bincode :: Encode ::
                encode(clock, encoder) ?; :: bincode :: Encode ::
                encode(msg_id, encoder) ?; :: bincode :: Encode ::
                encode(handoff, encoder) ?; core :: result :: Result :: Ok(())
            }, Self ::PutRsp { from, to, msg_id }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (3u32), encoder) ?
                ; :: bincode :: Encode :: encode(from, encoder) ?; :: bincode
                :: Encode :: encode(to, encoder) ?; :: bincode :: Encode ::
                encode(msg_id, encoder) ?; core :: result :: Result :: Ok(())
            }, Self ::GetReq { from, to, key, msg_id }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (4u32), encoder) ?
                ; :: bincode :: Encode :: encode(from, encoder) ?; :: bincode
                :: Encode :: encode(to, encoder) ?; :: bincode :: Encode ::
                encode(key, encoder) ?; :: bincode :: Encode ::
                encode(msg_id, encoder) ?; core :: result :: Result :: Ok(())
            }, Self ::GetRsp { from, to, key, values, msg_id }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (5u32), encoder) ?
                ; :: bincode :: Encode :: encode(from, encoder) ?; :: bincode
                :: Encode :: encode(to, encoder) ?; :: bincode :: Encode ::
                encode(key, encoder) ?; :: bincode :: Encode ::
                encode(values, encoder) ?; :: bincode :: Encode ::
                encode(msg_id, encoder) ?; core :: result :: Result :: Ok(())
            }, Self ::SyncKey { from, to, key, values }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (6u32), encoder) ?
                ; :: bincode :: Encode :: encode(from, encoder) ?; :: bincode
                :: Encode :: encode(to, encoder) ?; :: bincode :: Encode ::
                encode(key, encoder) ?; :: bincode :: Encode ::
                encode(values, encoder) ?; core :: result :: Result :: Ok(())
            }, Self ::PingReq { from, to }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (7u32), encoder) ?
                ; :: bincode :: Encode :: encode(from, encoder) ?; :: bincode
                :: Encode :: encode(to, encoder) ?; core :: result :: Result
                :: Ok(())
            }, Self ::PingRsp { from, to }
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (8u32), encoder) ?
                ; :: bincode :: Encode :: encode(from, encoder) ?; :: bincode
                :: Encode :: encode(to, encoder) ?; core :: result :: Result
                :: Ok(())
            },
        }
    }
}