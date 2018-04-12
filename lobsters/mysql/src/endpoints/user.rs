use futures::Future;
use my;
use my::prelude::*;
use trawler::UserId;

pub(crate) fn handle<F>(
    c: F,
    _acting_as: Option<UserId>,
    uid: UserId,
) -> Box<Future<Item = (my::Conn, bool), Error = my::errors::Error>>
where
    F: 'static + Future<Item = my::Conn, Error = my::errors::Error>,
{
    Box::new(
        c.and_then(move |c| {
            c.first_exec::<_, _, my::Row>(
                "SELECT  `users`.* FROM `users` \
                 WHERE `users`.`username` = ? \
                 ORDER BY `users`.`id` ASC LIMIT 1",
                (format!("user{}", uid),),
            )
        }).and_then(move |(c, user)| {
                let uid = user.unwrap().get::<u32, _>("id").unwrap();

                // most popular tag
                c.drop_exec(
                    "SELECT  `tags`.*, COUNT(*) AS `count` FROM `tags` \
                     INNER JOIN `taggings` ON `taggings`.`tag_id` = `tags`.`id` \
                     INNER JOIN `stories` ON `stories`.`id` = `taggings`.`story_id` \
                     WHERE `tags`.`inactive` = 0 \
                     AND `stories`.`user_id` = ? \
                     GROUP BY `tags`.`id` \
                     ORDER BY `count` desc LIMIT 1",
                    (uid,),
                ).and_then(move |c| {
                        c.drop_exec(
                            "SELECT  `keystores`.* \
                             FROM `keystores` \
                             WHERE `keystores`.`key` = ? \
                             ORDER BY `keystores`.`key` ASC LIMIT 1",
                            (format!("user:{}:stories_submitted", uid),),
                        )
                    })
                    .and_then(move |c| {
                        c.drop_exec(
                            "SELECT  `keystores`.* \
                             FROM `keystores` \
                             WHERE `keystores`.`key` = ? \
                             ORDER BY `keystores`.`key` ASC LIMIT 1",
                            (format!("user:{}:comments_posted", uid),),
                        )
                    })
                    .and_then(move |c| {
                        c.drop_exec(
                            "SELECT  1 AS one FROM `hats` \
                             WHERE `hats`.`user_id` = ? LIMIT 1",
                            (uid,),
                        )
                    })
            })
            .map(|c| (c, true)),
    )
}
