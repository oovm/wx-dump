attach database ?1 as MicroMsg;
select message.*, room.strNickName
-- n.UsrName
from MSG message
         -- left join Name2ID on n.rowid = MSG.TalkerId
         left join MicroMsg.Session room --
                   on room.strUsrName = message.StrTalker
order by Sequence desc
-- limit 10
;