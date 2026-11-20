from app.users import get_usr


def greet(user_id):
    return "hello, " + get_usr(user_id)
