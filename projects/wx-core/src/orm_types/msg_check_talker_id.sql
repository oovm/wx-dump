select --
       message.Sequence,
       message.StrTalker,
       talker.UsrName
from MSG message
         left join Name2ID talker on talker.rowid = message.TalkerId
order by Sequence desc
-- limit 10
;
