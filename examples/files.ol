import user;

enum Action {
    Delete->1;
    SecureDelete->2;
    Encrypt->3;
}

struct File {
    path->@string;
    size->@varuint64;
    content->@array[@uint8];
    owner->@option[User];
}

proc GetServerVersion() -> @int8;
proc GetFile(path->@string, after_action->@option[Action]) -> File;
proc DeleteFile(path->@string);
