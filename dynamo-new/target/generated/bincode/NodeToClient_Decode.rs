impl < __Context > :: bincode :: Decode < __Context > for NodeToClient
{
    fn decode < __D : :: bincode :: de :: Decoder < Context = __Context > >
    (decoder : & mut __D) ->core :: result :: Result < Self, :: bincode ::
    error :: DecodeError >
    {
        let variant_index = < u32 as :: bincode :: Decode ::< __D :: Context
        >>:: decode(decoder) ?; match variant_index
        {
            0u32 =>core :: result :: Result ::
            Ok(Self ::ClientPutRsp
            {
                key : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, request_id : :: bincode :: Decode ::< __D
                :: Context >:: decode(decoder) ?, client_addr : :: bincode ::
                Decode ::< __D :: Context >:: decode(decoder) ?,
            }), 1u32 =>core :: result :: Result ::
            Ok(Self ::ClientGetRsp
            {
                key : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?, request_id : :: bincode :: Decode ::< __D
                :: Context >:: decode(decoder) ?, values : :: bincode ::
                Decode ::< __D :: Context >:: decode(decoder) ?, metadata : ::
                bincode :: Decode ::< __D :: Context >:: decode(decoder) ?,
                client_addr : :: bincode :: Decode ::< __D :: Context >::
                decode(decoder) ?,
            }), variant =>core :: result :: Result ::
            Err(:: bincode :: error :: DecodeError :: UnexpectedVariant
            {
                found : variant, type_name : "NodeToClient", allowed : &::
                bincode :: error :: AllowedEnumVariants :: Range
                { min: 0, max: 1 }
            })
        }
    }
} impl < '__de, __Context > :: bincode :: BorrowDecode < '__de, __Context >
for NodeToClient
{
    fn borrow_decode < __D : :: bincode :: de :: BorrowDecoder < '__de,
    Context = __Context > > (decoder : & mut __D) ->core :: result :: Result <
    Self, :: bincode :: error :: DecodeError >
    {
        let variant_index = < u32 as :: bincode :: Decode ::< __D :: Context
        >>:: decode(decoder) ?; match variant_index
        {
            0u32 =>core :: result :: Result ::
            Ok(Self ::ClientPutRsp
            {
                key : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, request_id : :: bincode ::
                BorrowDecode ::< __D :: Context >:: borrow_decode(decoder) ?,
                client_addr : :: bincode :: BorrowDecode ::< __D :: Context
                >:: borrow_decode(decoder) ?,
            }), 1u32 =>core :: result :: Result ::
            Ok(Self ::ClientGetRsp
            {
                key : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, request_id : :: bincode ::
                BorrowDecode ::< __D :: Context >:: borrow_decode(decoder) ?,
                values : :: bincode :: BorrowDecode ::< __D :: Context >::
                borrow_decode(decoder) ?, metadata : :: bincode ::
                BorrowDecode ::< __D :: Context >:: borrow_decode(decoder) ?,
                client_addr : :: bincode :: BorrowDecode ::< __D :: Context
                >:: borrow_decode(decoder) ?,
            }), variant =>core :: result :: Result ::
            Err(:: bincode :: error :: DecodeError :: UnexpectedVariant
            {
                found : variant, type_name : "NodeToClient", allowed : &::
                bincode :: error :: AllowedEnumVariants :: Range
                { min: 0, max: 1 }
            })
        }
    }
}