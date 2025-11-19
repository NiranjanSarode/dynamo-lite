impl :: bincode :: Encode for VersionedValues
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.versions, encoder) ?; core ::
        result :: Result :: Ok(())
    }
}