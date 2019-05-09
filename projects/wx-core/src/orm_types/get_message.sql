ATTACH DATABASE ?1 AS MicroMsg;
select message.*, room.strNickName
--        n.UsrName
from MSG message
         --          JOIN main.Name2ID n ON n.rowid  = m.TalkerId
         left join MicroMsg.Session room --
              on room.strUsrName = message.StrTalker
order by Sequence desc
-- limit 10
;