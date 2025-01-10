attach database ?1 as MicroMsg;
select --
       message.*,
       rooms.strNickName         as RoomName,
       get_sender_id(BytesExtra) as SenderId,
       senders.NickName   as SenderName
from MSG message
         left join MicroMsg.Session rooms --
                   on rooms.strUsrName = message.StrTalker
         left join MicroMsg.Contact senders --
                   on senders.UserName = SenderId
order by Sequence desc
-- limit 10
;
