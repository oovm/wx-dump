select m.*, n.UsrName
from main.MSG m
         JOIN main.Name2ID n ON n.rowid - 1 = m.TalkerId
order by Sequence desc
limit 10
;