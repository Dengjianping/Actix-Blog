## About this project(Experimenal)
I' a new rustacean, using actix-web to build a personal blog right now, and trying to use the features in rust(learning by practice), so maybe I'll modify these code frequently(but based on the latest stable rust).

## Thanks for for the resources
1. Bootstrap 4.x.
2. Blog templates from [Start Bootstrap - Clean Blog](https://github.com/BlackrockDigital/startbootstrap-clean-blog).
3. Admin templates from [Start Bootstrap - SB Admin](https://github.com/BlackrockDigital/startbootstrap-sb-admin).
4. [Showdown](https://github.com/showdownjs/showdown) for renderring markdown.
5. ...

## Requirements
1. stable rust. (>=1.30, begin to support proc_macro_attribute in stable version, I didn't compile on nightly or beta)
2. Postgresql. (10.5, I didn't try on 9.x or 11.x)
3. Tera for template renderring
4. ...

## Deployment(do not try now)
1. Follow the official [diesel](diesel.rs) doc to configure database.
2. Create some related tables in pg.

```sh
# create user table

# create post table

# ...

```


## Glance
![main page](samples/blog_page.JPG)
![admin page](samples/admin_page.JPG)

## Issues
1. Still having problem on renderring markdown, like programming-lang code section.
2. Cookies handling.
3. Password resetting.
4. No comments system(but incoming).
5. No 404 page.
6. No tests.
7. Lots of issues I don't post here.