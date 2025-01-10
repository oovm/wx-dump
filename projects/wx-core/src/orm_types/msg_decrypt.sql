attach database ?1 as MicroMsg;
select --
       message.*,
       get_sender_id(BytesExtra) as SenderId,
       room.strNickName
from MSG message
         left join MicroMsg.Session room --
                   on room.strUsrName = message.StrTalker
order by Sequence desc
-- limit 10
;
