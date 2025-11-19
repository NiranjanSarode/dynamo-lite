impl < __Context > :: bincode :: Decode < __Context > for NodeToNode
{
    fn decode < __D : :: bincode :: de :: Decoder < Context = __Context > >
    (decoder : & mut __D) ->core :: result :: Result < Self, :: bincode ::
    error :: DecodeError >
    {
        let variant_index = < u32 as :: bincode :: Decode ::< __D :: Context
        >>:: decode(decoder) ?; match variant_index
        {
            0u32 =>core :: result :: Result ::
            Ok(Self ::ForwardClientPut
            {
                coordinator : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, key : :: bincode :: Decode ::< __D ::
                Context >:: decode(decoder) ?, value : :: bincode :: Decode
                ::< __D :: Context >:: decode(decoder) ?, metadata : ::
                bincode :: Decode ::< __D :: Context >:: decode(decoder) ?,
                client_addr : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, request_id : :: bincode :: Decode ::< __D
                :: Context >:: decode(decoder) ?,
            }), 1u32 =>core :: result :: Result ::
            Ok(Self ::ForwardClientGet
            {
                coordinator : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, key : :: bincode :: Decode ::< __D ::
                Context >:: decode(decoder) ?, client_addr : :: bincode ::
                Decode ::< __D :: Context >:: decode(decoder) ?, request_id :
                :: bincode :: Decode ::< __D :: Context >:: decode(decoder) ?,
            }), 2u32 =>core :: result :: Result ::
            Ok(Self ::PutReq
            {
                from : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, to : :: bincode :: Decode ::< __D ::
                Context >:: decode(decoder) ?, key : :: bincode :: Decode ::<
                __D :: Context >:: decode(decoder) ?, value : :: bincode ::
                Decode ::< __D :: Context >:: decode(decoder) ?, clock : ::
                bincode :: Decode ::< __D :: Context >:: decode(decoder) ?,
                msg_id : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, handoff : :: bincode :: Decode ::< __D ::
                Context >:: decode(decoder) ?,
            }), 3u32 =>core :: result :: Result ::
            Ok(Self ::PutRsp
            {
                from : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, to : :: bincode :: Decode ::< __D ::
                Context >:: decode(decoder) ?, msg_id : :: bincode :: Decode
                ::< __D :: Context >:: decode(decoder) ?,
            }), 4u32 =>core :: result :: Result ::
            Ok(Self ::GetReq
            {
                from : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, to : :: bincode :: Decode ::< __D ::
                Context >:: decode(decoder) ?, key : :: bincode :: Decode ::<
                __D :: Context >:: decode(decoder) ?, msg_id : :: bincode ::
                Decode ::< __D :: Context >:: decode(decoder) ?,
            }), 5u32 =>core :: result :: Result ::
            Ok(Self ::GetRsp
            {
                from : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, to : :: bincode :: Decode ::< __D ::
                Context >:: decode(decoder) ?, key : :: bincode :: Decode ::<
                __D :: Context >:: decode(decoder) ?, values : :: bincode ::
                Decode ::< __D :: Context >:: decode(decoder) ?, msg_id : ::
                bincode :: Decode ::< __D :: Context >:: decode(decoder) ?,
            }), 6u32 =>core :: result :: Result ::
            Ok(Self ::SyncKey
            {
                from : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, to : :: bincode :: Decode ::< __D ::
                Context >:: decode(decoder) ?, key : :: bincode :: Decode ::<
                __D :: Context >:: decode(decoder) ?, values : :: bincode ::
                Decode ::< __D :: Context >:: decode(decoder) ?,
            }), 7u32 =>core :: result :: Result ::
            Ok(Self ::PingReq
            {
                from : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, to : :: bincode :: Decode ::< __D ::
                Context >:: decode(decoder) ?,
            }), 8u32 =>core :: result :: Result ::
            Ok(Self ::PingRsp
            {
                from : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, to : :: bincode :: Decode ::< __D ::
                Context >:: decode(decoder) ?,
            }), 9u32 =>core :: result :: Result ::
            Ok(Self ::AddNode
            {
                from : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, to : :: bincode :: Decode ::< __D ::
                Context >:: decode(decoder) ?, new_node : :: bincode :: Decode
                ::< __D :: Context >:: decode(decoder) ?,
            }), 10u32 =>core :: result :: Result ::
            Ok(Self ::AddNodeAck
            {
                from : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, to : :: bincode :: Decode ::< __D ::
                Context >:: decode(decoder) ?, new_node : :: bincode :: Decode
                ::< __D :: Context >:: decode(decoder) ?,
            }), variant =>core :: result :: Result ::
            Err(:: bincode :: error :: DecodeError :: UnexpectedVariant
            {
                found : variant, type_name : "NodeToNode", allowed : &::
                bincode :: error :: AllowedEnumVariants :: Range
                { min: 0, max: 10 }
            })
        }
    }
} impl < '__de, __Context > :: bincode :: BorrowDecode < '__de, __Context >
for NodeToNode
{
    fn borrow_decode < __D : :: bincode :: de :: BorrowDecoder < '__de,
    Context = __Context > > (decoder : & mut __D) ->core :: result :: Result <
    Self, :: bincode :: error :: DecodeError >
    {
        let variant_index = < u32 as :: bincode :: Decode ::< __D :: Context
        >>:: decode(decoder) ?; match variant_index
        {
            0u32 =>core :: result :: Result ::
            Ok(Self ::ForwardClientPut
            {
                coordinator : :: bincode :: BorrowDecode ::< __D :: Context
                >:: borrow_decode(decoder) ?, key : :: bincode :: BorrowDecode
                ::< __D :: Context >:: borrow_decode(decoder) ?, value : ::
                bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, metadata : :: bincode ::
                BorrowDecode ::< __D :: Context >:: borrow_decode(decoder) ?,
                client_addr : :: bincode :: BorrowDecode ::< __D :: Context
                >:: borrow_decode(decoder) ?, request_id : :: bincode ::
                BorrowDecode ::< __D :: Context >:: borrow_decode(decoder) ?,
            }), 1u32 =>core :: result :: Result ::
            Ok(Self ::ForwardClientGet
            {
                coordinator : :: bincode :: BorrowDecode ::< __D :: Context
                >:: borrow_decode(decoder) ?, key : :: bincode :: BorrowDecode
                ::< __D :: Context >:: borrow_decode(decoder) ?, client_addr :
                :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, request_id : :: bincode ::
                BorrowDecode ::< __D :: Context >:: borrow_decode(decoder) ?,
            }), 2u32 =>core :: result :: Result ::
            Ok(Self ::PutReq
            {
                from : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, to : :: bincode :: BorrowDecode ::<
                __D :: Context >:: borrow_decode(decoder) ?, key : :: bincode
                :: BorrowDecode ::< __D :: Context >:: borrow_decode(decoder)
                ?, value : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, clock : :: bincode :: BorrowDecode
                ::< __D :: Context >:: borrow_decode(decoder) ?, msg_id : ::
                bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, handoff : :: bincode :: BorrowDecode
                ::< __D :: Context >:: borrow_decode(decoder) ?,
            }), 3u32 =>core :: result :: Result ::
            Ok(Self ::PutRsp
            {
                from : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, to : :: bincode :: BorrowDecode ::<
                __D :: Context >:: borrow_decode(decoder) ?, msg_id : ::
                bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?,
            }), 4u32 =>core :: result :: Result ::
            Ok(Self ::GetReq
            {
                from : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, to : :: bincode :: BorrowDecode ::<
                __D :: Context >:: borrow_decode(decoder) ?, key : :: bincode
                :: BorrowDecode ::< __D :: Context >:: borrow_decode(decoder)
                ?, msg_id : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?,
            }), 5u32 =>core :: result :: Result ::
            Ok(Self ::GetRsp
            {
                from : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, to : :: bincode :: BorrowDecode ::<
                __D :: Context >:: borrow_decode(decoder) ?, key : :: bincode
                :: BorrowDecode ::< __D :: Context >:: borrow_decode(decoder)
                ?, values : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, msg_id : :: bincode :: BorrowDecode
                ::< __D :: Context >:: borrow_decode(decoder) ?,
            }), 6u32 =>core :: result :: Result ::
            Ok(Self ::SyncKey
            {
                from : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, to : :: bincode :: BorrowDecode ::<
                __D :: Context >:: borrow_decode(decoder) ?, key : :: bincode
                :: BorrowDecode ::< __D :: Context >:: borrow_decode(decoder)
                ?, values : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?,
            }), 7u32 =>core :: result :: Result ::
            Ok(Self ::PingReq
            {
                from : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, to : :: bincode :: BorrowDecode ::<
                __D :: Context >:: borrow_decode(decoder) ?,
            }), 8u32 =>core :: result :: Result ::
            Ok(Self ::PingRsp
            {
                from : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, to : :: bincode :: BorrowDecode ::<
                __D :: Context >:: borrow_decode(decoder) ?,
            }), 9u32 =>core :: result :: Result ::
            Ok(Self ::AddNode
            {
                from : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, to : :: bincode :: BorrowDecode ::<
                __D :: Context >:: borrow_decode(decoder) ?, new_node : ::
                bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?,
            }), 10u32 =>core :: result :: Result ::
            Ok(Self ::AddNodeAck
            {
                from : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, to : :: bincode :: BorrowDecode ::<
                __D :: Context >:: borrow_decode(decoder) ?, new_node : ::
                bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?,
            }), variant =>core :: result :: Result ::
            Err(:: bincode :: error :: DecodeError :: UnexpectedVariant
            {
                found : variant, type_name : "NodeToNode", allowed : &::
                bincode :: error :: AllowedEnumVariants :: Range
                { min: 0, max: 10 }
            })
        }
    }
}