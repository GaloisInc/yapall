/* Derived from chess-aces program with MIT license (reproduced below).
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the “Software”), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

#define _XOPEN_SOURCE 700

#include <ctype.h>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <grp.h>
#include <libgen.h>
#include <limits.h>
#include <malloc.h>
#include <netinet/in.h>
#include <pwd.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

#include <ftw.h>

typedef struct cmd {
  int argc;
  int max_argv;
  char **argv;

  FILE *out;
  FILE *in;

  int close_out;
  int close_in;

  struct cmd *next;
} cmd;

typedef struct file_info {
  struct dirent d;
  struct stat st;
  struct file_info *next;
} file_info;

typedef struct env {
  char *name;
  char *value;

  struct env *next;
} env;

#define CMD 0
#define SPACE 1
#define END 2
#define NEWCMD 3
#define INITCMD 4
#define ARG 5
#define OPENQUOTE_ARG 6
#define SUBCMD 7
#define PIPE 8
#define EXEC_TO_NEWCMD 9
#define ARROW 10
#define ERROR -1

int last_exit_code;
char *home = "/home/chess";
char *user = "chess";
char *hostname = "whatever";
char *tokenfile = "/token";

static volatile int hit_ctrl_c = 0;

FILE *default_in;
FILE *default_out;

FILE *curin;
FILE *curout;

int curin_needs_closing = 0;
int curout_needs_closing = 0;

env *env_vars = NULL;

uid_t current_euid;
gid_t current_egid;
uid_t saved_euid;
gid_t saved_egid;

cmd *rcomm = NULL;
int rvfl = 0;

void sigintHandler(int info) {

  if (hit_ctrl_c) {
    hit_ctrl_c = 0;
    return;
  } else {
    exit(0);
  }
}

void free_cmd(cmd *c) {
  int i;
  if (!c) {
    return;
  }

  if (c->argv) {
    for (i = 0; i < c->argc; i++) {
      free(c->argv[i]);

      c->argv[i] = NULL;
    }

    free(c->argv);
    c->argv = NULL;
  }

  free(c);

  return;
}

void print_cmd(cmd *c) {
  int i = 0;

  if (!c) {
    return;
  }

  for (i = 0; i < c->argc; i++) {
    printf("%s ", c->argv[i]);
  }

  printf("\n");
  return;
}

int droppriv() {
  struct passwd *pw = NULL;

  pw = getpwnam("chess");

  if (pw == NULL) {
    fprintf(stderr, "getpwnam(): %s\n", strerror(errno));
    exit(1);
  }

  if (setegid(pw->pw_gid)) {
    fprintf(stderr, "setegid(): %s\n", strerror(errno));
    exit(1);
  }

  if (seteuid(pw->pw_uid)) {
    fprintf(stderr, "seteuid(): %s\n", strerror(errno));
    exit(1);
  }

  saved_euid = current_euid;
  saved_egid = current_egid;

  current_euid = pw->pw_uid;
  current_egid = pw->pw_gid;

  return 0;
}

int uppriv() {
  if (seteuid(saved_euid)) {
    fprintf(stderr, "seteuid(): %s\n", strerror(errno));
    exit(1);
  }

  if (setegid(saved_egid)) {
    fprintf(stderr, "setegid(): %s\n", strerror(errno));
    exit(1);
  }

  current_euid = saved_euid;
  current_egid = saved_egid;

  return 0;
}

int append_env(env *nv) {
  if (!nv) {
    return -1;
  }

  if (!env_vars) {
    env_vars = nv;
  } else {
    nv->next = env_vars;
    env_vars = nv;
  }

  return 0;
}

env *makevar(char *name, char *value) {
  env *nv = NULL;

  if (!name || !value) {
    return NULL;
  }

  nv = malloc(sizeof(env));

  if (!nv) {
    return NULL;
  }

  memset(nv, 0, sizeof(env));

  nv->name = strdup(name);
  nv->value = strdup(value);

  return nv;
}

env *getenvvar(char *name) {
  env *walker = NULL;

  if (!name) {
    return NULL;
  }

  walker = env_vars;

  while (walker) {
    if (!strcmp(walker->name, name)) {
      return walker;
    }

    walker = walker->next;
  }

  return NULL;
}

char *getenvvalue(char *name) {
  env *nv;

  if (!name) {
    return NULL;
  }

  nv = getenvvar(name);

  if (nv) {
    return nv->value;
  }

  return NULL;
}

int setenvvar(char *name, char *value) {
  env *nv = NULL;

  if (!name || !value) {
    return -1;
  }

  nv = getenvvar(name);

  if (nv) {
    free(nv->value);
    nv->value = strdup(value);
  } else {
    return append_env(makevar(name, value));
  }

  return 0;
}

int init_env(char *home, char *pwd, char *user) {
  if (!home || !pwd || !user) {
    return -1;
  }

  append_env(makevar("HOME", home));
  append_env(makevar("PWD", pwd));
  append_env(makevar("USER", user));

  chdir(home);

  return 0;
}

int handle_cd(cmd *command) {
  struct stat st;

  if (command == NULL) {
    return -1;
  }

  if (command->argc > 2) {
    fprintf(command->out, "cd: too many arguments\n");
    return -1;
  }

  if (command->argc == 1) {
    command->argv[1] = home;
  }

  if (lstat(command->argv[1], &st)) {
    fprintf(command->out, "cd: %s: No such file or directory\n",
            command->argv[1]);
    return -1;
  }

  if (chdir(command->argv[1])) {
    fprintf(command->out, "cd: %s: %s\n", command->argv[1], strerror(errno));
    return errno;
  }

  setenvvar("PWD", command->argv[1]);

  return 1;
}

int read_echo_loop(cmd *command) {
  char c;
  int retval;
  hit_ctrl_c = 1;

  if (!command) {
    return -1;
  }

  while (hit_ctrl_c) {
    retval = fread(&c, 1, 1, command->in);

    if (retval <= 0) {
      hit_ctrl_c = 0;
      return retval;
    }

    fprintf(command->out, "%c", c);
  }

  return 0;
}

int can_access_file(char *fn, mode_t access) {
  struct stat st;
  struct passwd *pw;
  uid_t uid;

  if (fn == NULL) {
    return 0;
  }

  if (lstat(fn, &st)) {
    return 0;
  }

  pw = getpwnam(user);

  if (pw == NULL) {
    return 0;
  }

  uid = pw->pw_uid;

  if (uid == st.st_uid) {
    return 1;
  }

  if (access & st.st_mode) {
    return 1;
  }

  return 0;
}

int handle_cat(cmd *command) {
  struct stat st;
  int len = 0;
  char *data = NULL;
  int fd = 0;
  int i = 0;

  if (!command) {
    return -1;
  }

  if (command->argc == 1) {
    i = read_echo_loop(command);

    return i;
  }

  for (i = 1; i < command->argc; i++) {
    if (lstat(command->argv[i], &st)) {
      fprintf(command->out, "cat: %s: %s\n", command->argv[i], strerror(errno));
      continue;
    }

    if (S_ISDIR(st.st_mode)) {
      fprintf(command->out, "cat: %s: Is a directory\n", command->argv[i]);
      continue;
    }

    if (!can_access_file(command->argv[i], S_IROTH)) {
      fprintf(command->out, "cat: %s: permission failed\n", command->argv[i]);
      continue;
    }

    fd = open(command->argv[i], O_RDONLY);

    if (fd <= 0) {
      fprintf(command->out, "cat: %s: '%s'\n", command->argv[i],
              strerror(errno));
      continue;
    }

    len = lseek(fd, 0, SEEK_END);
    lseek(fd, 0, SEEK_SET);

    data = malloc(len);

    if (!data) {
      fprintf(command->out, "malloc() fail\n");
      close(fd);
      return -1;
    }

    memset(data, 0, len);
    read(fd, data, len);

    close(fd);

    for (int j = 0; j < len; j++) {
      fprintf(command->out, "%c", data[j]);
    }

    free(data);
    data = NULL;
  }

  return 0;
}

int cp_file(char *src, char *dest) {
  int in_fd, out_fd;
  int len;
  char *data;
  struct stat st;

  if (!src || !dest) {
    return -1;
  }

  if (lstat(src, &st)) {
    return -1;
  }

  droppriv();

  if (!can_access_file(src, S_IROTH)) {
    uppriv();
    return -1;
  }

  in_fd = open(src, O_RDONLY);

  if (in_fd < 0) {
    uppriv();
    return -1;
  }

  out_fd = open(dest, O_WRONLY | O_CREAT | O_TRUNC, st.st_mode);

  if (out_fd < 0) {
    close(in_fd);
    uppriv();
    return -1;
  }

  len = lseek(in_fd, 0, SEEK_END);
  lseek(in_fd, 0, SEEK_SET);

  data = malloc(len);

  if (!data) {
    close(in_fd);
    close(out_fd);
    uppriv();
    return -1;
  }

  read(in_fd, data, len);

  close(in_fd);

  write(out_fd, data, len);

  close(out_fd);

  free(data);
  data = NULL;

  uppriv();
  return 0;
}

int copy_directory(char *src, char *dest, int v_flag, cmd *command) {
  struct stat src_st;
  struct stat dest_st;
  DIR *dir = NULL;
  struct dirent *ent = NULL;
  struct stat st;
  char fullname_src[PATH_MAX];
  char fullname_dest[PATH_MAX];
  char *fp_dest;
  char *fp_src;

  if (!src || !dest || !command) {
    return -1;
  }

  if (!strcmp(dest, ".") || !strcmp(dest, "..")) {
    return 0;
  }

  if (!strcmp(src, ".") || !strcmp(src, "..")) {
    return 0;
  }

  if (lstat(src, &src_st)) {
    return -1;
  }

  if (!S_ISDIR(src_st.st_mode)) {
    return -1;
  }

  droppriv();

  if (lstat(dest, &dest_st)) {
    if (mkdir(dest, src_st.st_mode)) {
      uppriv();
      return -1;
    }
  }

  dir = opendir(src);

  if (!dir) {
    uppriv();
    return -1;
  }

  fp_dest = realpath(dest, NULL);

  if (!fp_dest) {
    uppriv();
    return -1;
  }

  fp_src = realpath(src, NULL);

  if (!fp_src) {
    free(fp_dest);
    fp_dest = NULL;
    uppriv();
    return -1;
  }

  while ((ent = readdir(dir)) != NULL) {
    if (!strcmp(ent->d_name, ".") || !strcmp(ent->d_name, "..")) {
      continue;
    }

    snprintf(fullname_src, PATH_MAX, "%s/%s", fp_src, ent->d_name);
    snprintf(fullname_dest, PATH_MAX, "%s/%s", fp_dest, ent->d_name);

    if (lstat(fullname_src, &st)) {
      fprintf(command->out, "cp: '%s': %s\n", fullname_src, strerror(errno));
      continue;
    }

    if (!S_ISDIR(st.st_mode)) {
      if (cp_file(fullname_src, fullname_dest)) {
        fprintf(command->out, "cp: failed to copy '%s': %s\n", fullname_dest,
                strerror(errno));
        continue;
      }

      if (v_flag) {
        fprintf(command->out, "%s -> %s\n", fullname_src, fullname_dest);
      }
    } else {

      copy_directory(fullname_src, fullname_dest, v_flag, command);
    }
  }

  free(fp_dest);
  free(fp_src);

  fp_dest = NULL;
  fp_src = NULL;

  uppriv();
  return 0;
}

int handle_cp(cmd *command) {

  int r_flag = 0;

  int v_flag = 0;

  if (!command) {
    return -1;
  }

  char *dest = NULL;
  struct stat dest_st;

  int index = 0;
  int file_arg_count;
  int c = 0;
  int len;

  int in_fd, out_fd;
  char *data;
  char out_filename[PATH_MAX];
  struct stat st;

  if (!command) {
    return -1;
  }

  opterr = 0;
  optind = 0;

  while ((c = getopt(command->argc, command->argv, "rRhv")) != -1) {
    switch (c) {
    case 'r':
      r_flag = 1;
      break;
    case 'R':
      r_flag = 1;
      break;
    case 'h':
      fprintf(command->out,
              "cp [OPTION] SOURCE DEST\n\t-r -R Copy recursively\n\t-h Print "
              "this help\n\t-v Verbose mode\n");
      return 0;
      break;
    case 'v':
      v_flag = 1;
      break;
    case '?':
      if (isprint(optopt))
        fprintf(command->out, "Unknown option `-%c'.\n", optopt);
      else
        fprintf(command->out, "Unknown option character `\\x%x'.\n", optopt);

      return -1;
    default:
      return -1;
    }
  }

  c = 0;

  file_arg_count = command->argc - optind;

  if (file_arg_count <= 0) {
    fprintf(command->out, "cp: missing file operand\n");
    return -1;
  } else if (file_arg_count == 1) {
    fprintf(command->out, "cp: missing destination file operand after '%s'\n",
            command->argv[optind]);
    return -1;
  }

  dest = command->argv[command->argc - 1];

  if (file_arg_count >= 3) {
    if (lstat(dest, &dest_st)) {
      fprintf(command->out, "cp: target '%s' is not a directory\n", dest);
      return -1;
    }

    if (!S_ISDIR(dest_st.st_mode)) {
      fprintf(command->out, "cp: target '%s' is not a directory\n", dest);
      return -1;
    }
  }

  for (index = optind; index < command->argc - 1; index++) {

    if (lstat(command->argv[index], &st)) {
      fprintf(command->out, "cp: cannot stat '%s': %s\n", command->argv[index],
              strerror(errno));
      continue;
    }

    if (S_ISDIR(st.st_mode)) {

      if (!r_flag) {
        fprintf(command->out, "cp: -r not specified; omitting directory '%s'\n",
                command->argv[index]);
        continue;
      }

      copy_directory(command->argv[index], dest, v_flag, command);

      continue;
    }

    droppriv();
    in_fd = open(command->argv[index], O_RDONLY);

    if (in_fd < 0) {
      fprintf(command->out, "cp: failed to open '%s': %s\n",
              command->argv[index], strerror(errno));
      uppriv();
      continue;
    }

    memset(out_filename, 0, PATH_MAX);

    if (!access(dest, W_OK)) {
      if (lstat(dest, &dest_st)) {
        fprintf(command->out, "cp: failed to stat '%s': %s\n", dest,
                strerror(errno));
        close(in_fd);
        uppriv();
        continue;
      }

      if (S_ISDIR(dest_st.st_mode)) {
        data = realpath(dest, NULL);

        if (!data) {
          fprintf(command->out, "cp: failed to get path '%s': %s\n", dest,
                  strerror(errno));
          close(in_fd);
          uppriv();
          continue;
        }

        char *fn = strrchr(command->argv[index], '/');

        if (fn == NULL) {
          fn = command->argv[index];
        } else {
          fn += 1;
        }

        snprintf(out_filename, PATH_MAX, "%s/%s", data, fn);

        free(data);
        data = NULL;
      } else {
        snprintf(out_filename, PATH_MAX, "%s", dest);
      }

    } else {
      snprintf(out_filename, PATH_MAX, "%s", dest);
    }

    out_fd = open(out_filename, O_WRONLY | O_CREAT | O_TRUNC, st.st_mode);

    if (out_fd < 0) {
      fprintf(command->out, "cp: failed to copy '%s': %s\n", out_filename,
              strerror(errno));
      close(in_fd);
      uppriv();
      continue;
    }

    len = lseek(in_fd, 0, SEEK_END);
    lseek(in_fd, 0, SEEK_SET);

    data = malloc(len);

    if (!data) {
      fprintf(command->out, "cp: malloc() fail: %s\n", strerror(errno));
      close(in_fd);
      close(out_fd);
      uppriv();
      continue;
    }

    read(in_fd, data, len);

    close(in_fd);

    write(out_fd, data, len);

    close(out_fd);

    free(data);
    data = NULL;

    if (v_flag) {
      fprintf(command->out, "%s -> %s\n", command->argv[index], out_filename);
    }

    uppriv();
  }

  return 0;
}

int handle_env(cmd *command) {
  env *walker = NULL;

  if (command == NULL) {
    return -1;
  }

  walker = env_vars;

  while (walker) {
    fprintf(command->out, "%s=%s\n", walker->name, walker->value);

    walker = walker->next;
  }

  return 0;
}

char *convert_escapes(char *in) {
  char *nl = NULL;
  int in_index = 0;
  int nl_index = 0;
  int ml = 0;
  char c;

  if (!in) {
    return NULL;
  }

  ml = strlen(in);

  nl = malloc(ml * 2);

  if (!nl) {
    return NULL;
  }

  memset(nl, 0, ml * 2);

  while (in_index < ml) {
    if (in[in_index] == '\\') {
      if (in_index + 1 >= ml) {
        nl[nl_index] = in[in_index];
        return nl;
      }

      in_index++;

      switch (in[in_index]) {
      case '\\':
        nl[nl_index++] = in[in_index++];
        break;
      case 'a':
        c = '\x07';
        break;
      case 'b':
        c = '\x08';
        break;
      case 'f':
        c = '\x0c';
        break;
      case 'n':
        c = '\x0a';
        break;
      case 'r':
        c = '\x0d';
        break;
      case 't':
        c = '\x09';
        break;
      case 'v':
        c = '\x0b';
        break;
      case 'x':
        if (in_index + 2 >= ml) {
          nl[nl_index++] = '\\';
          nl[nl_index++] = in[in_index++];
          continue;
        } else {

          switch (in[in_index + 1]) {
          case 'a':
          case 'b':
          case 'c':
          case 'd':
          case 'e':
          case 'f':
            c = in[in_index + 1] - 0x57;
            break;
          case 'A':
          case 'B':
          case 'C':
          case 'D':
          case 'E':
          case 'F':
            c = in[in_index + 1] - 0x37;
            break;
          case '0':
          case '1':
          case '2':
          case '3':
          case '4':
          case '5':
          case '6':
          case '7':
          case '8':
          case '9':
            c = in[in_index + 1] - 0x30;
            break;
          default:
            nl[nl_index++] = '\\';
            nl[nl_index++] = 'x';
            nl[nl_index++] = in[in_index + 1];
            in_index += 2;
            continue;
            break;
          }

          c <<= 4;

          switch (in[in_index + 2]) {
          case 'a':
          case 'b':
          case 'c':
          case 'd':
          case 'e':
          case 'f':
            c |= in[in_index + 2] - 0x57;
            break;
          case 'A':
          case 'B':
          case 'C':
          case 'D':
          case 'E':
          case 'F':
            c |= in[in_index + 2] - 0x37;
            break;
          case '0':
          case '1':
          case '2':
          case '3':
          case '4':
          case '5':
          case '6':
          case '7':
          case '8':
          case '9':
            c |= in[in_index + 2] - 0x30;
            break;
          default:
            nl[nl_index++] = '\\';
            nl[nl_index++] = in[in_index++];
            nl[nl_index++] = in[in_index++];
            nl[nl_index++] = in[in_index++];
            continue;
            break;
          }

          nl[nl_index++] = c;
          in_index += 3;
        }
        break;
      default:
        nl[nl_index++] = '\\';
        nl[nl_index++] = in[in_index++];
        continue;
        break;
      }

    } else {
      nl[nl_index] = in[in_index];
      nl_index++;
      in_index++;
    }
  }

  return nl;
}

int handle_echo(cmd *command) {
  int c;
  int n_flag = 0;
  int e_flag = 0;
  int index;
  char *t;

  if (command == NULL) {
    return -1;
  }

  opterr = 0;
  optind = 0;

  while ((c = getopt(command->argc, command->argv, "nhe")) != -1) {
    switch (c) {
    case 'n':
      n_flag = 1;
      break;
    case 'h':
      fprintf(command->out,
              "echo [OPTION]\n\t-n do not output the trailing newline\n\t-e "
              "enable interpretation of backslash escapes\n");
      return 0;
      break;
    case 'e':
      e_flag = 1;
      break;
    case '?':
      if (isprint(optopt)) {
        fprintf(command->out, "Unknown option `-%c'.\n", optopt);
      } else {
        fprintf(command->out, "Unknown option character `\\x%x'.\n", optopt);
      }

      return -1;
    default:
      return -1;
    }
  }

  for (index = optind; index < command->argc; index++) {
    if (e_flag) {
      t = convert_escapes(command->argv[index]);

      if (t) {
        free(command->argv[index]);
        command->argv[index] = t;
      }
    }

    fprintf(command->out, "%s", command->argv[index]);

    if (index + 1 < optind) {
      fprintf(command->out, " ");
    }
  }

  if (!n_flag) {
    fprintf(command->out, "\n");
  }

  return 0;
}

int handle_date(cmd *command) {
  char *tm = NULL;
  time_t now;

  if (!command) {
    return -1;
  }

  now = time(NULL);

  tm = ctime(&now);

  if (!tm) {
    tm = "unknown\n";
  }

  fprintf(command->out, "%s", tm);

  return 0;
}

int handle_cmp(cmd *command) {
  int b_flag = 0;

  int filea_index = 0;
  int fileb_index = 0;

  int filea_len = 0;
  int fileb_len = 0;

  char *skips = NULL;
  char *skt = NULL;
  char *limit = NULL;

  char *filea = NULL;
  char *fileb = NULL;

  struct stat sta;
  struct stat stb;

  char *filea_data = NULL;
  char *fileb_data = NULL;

  int filea_fd;
  int fileb_fd;

  int max = 0;
  int total = 0;

  int c;
  int index;

  if (!command) {
    return -1;
  }

  opterr = 0;
  optind = 0;

  while ((c = getopt(command->argc, command->argv, "bi:n:h")) != -1) {
    switch (c) {
    case 'b':
      b_flag = 1;
      break;
    case 'i':
      skips = strdup(optarg);
      break;
    case 'n':
      limit = strdup(optarg);
      break;
    case 'h':
      fprintf(command->out,
              "cmp [OPTION]\n\t-b print differing bytes\n\t-i SKIP1:SKIP2 skip "
              "first SKIP1 bytes of FILE1 and first SKIP2 bytes of FILE2\n\t-n "
              "LIMIT compare at most LIMIT bytes\n");
      return 0;
      break;
    case '?':
      if (optopt == 'i') {
        fprintf(command->out, "cmp: option requires an argument -- 'i'\n");
      } else if (optopt == 'n') {
        fprintf(command->out, "cmp: option requires an argument -- 'n'\n");
      } else if (isprint(optopt)) {
        fprintf(command->out, "Unknown option `-%c'.\n", optopt);
      } else {
        fprintf(command->out, "Unknown option character `\\x%x'.\n", optopt);
      }

      if (limit) {
        free(limit);
        limit = NULL;
      }
      return -1;
    default:
      if (limit) {
        free(limit);
        limit = NULL;
      }
      return -1;
    }
  }

  if (skips) {
    skt = strchr(skips, ':');

    if (skt) {
      skt[0] = '\x00';

      filea_index = strtol(skips, NULL, 10);
      fileb_index = strtol(skt + 1, NULL, 10);
    } else {
      filea_index = strtol(skips, NULL, 10);
    }
  }

  if (limit) {
    max = strtol(limit, NULL, 10);

    free(limit);
    limit = NULL;
  }

  index = command->argc - optind;

  if (index == 0) {
    fprintf(command->out, "cmp: missing file operand\n");
    return -1;
  } else if (index == 1) {
    fprintf(command->out, "cmp: not doing stdin yet\n");
    filea = command->argv[optind];

    if (lstat(filea, &sta)) {
      fprintf(command->out, "cmp: '%s': %s\n", filea, strerror(errno));
      return -1;
    }

    return -1;
  } else if (index == 2) {
    filea = command->argv[optind];
    fileb = command->argv[optind + 1];

    if (lstat(filea, &sta)) {
      fprintf(command->out, "cmp: '%s': %s\n", filea, strerror(errno));
      return -1;
    }

    if (lstat(fileb, &stb)) {
      fprintf(command->out, "cmp: '%s': %s\n", fileb, strerror(errno));
      return -1;
    }

    if (!can_access_file(filea, S_IROTH)) {
      fprintf(command->out, "cmp: failed '%s' permission denied\n", filea);
      return -1;
    }

    if (!can_access_file(fileb, S_IROTH)) {
      fprintf(command->out, "cmp: failed '%s' permission denied\n", fileb);
      return -1;
    }

    filea_fd = open(filea, O_RDONLY);

    if (filea_fd < 0) {
      fprintf(command->out, "cmp: '%s': %s\n", filea, strerror(errno));
      return -1;
    }

    fileb_fd = open(fileb, O_RDONLY);

    if (fileb_fd < 0) {
      close(filea_fd);
      fprintf(command->out, "cmp: '%s': %s\n", fileb, strerror(errno));
      return -1;
    }

    filea_len = lseek(filea_fd, 0, SEEK_END);
    lseek(filea_fd, 0, SEEK_SET);

    filea_data = malloc(filea_len);

    if (!filea_data) {
      close(filea_fd);
      close(fileb_fd);
      fprintf(command->out, "cmp: malloc() fail\n");
      return -1;
    }

    fileb_len = lseek(fileb_fd, 0, SEEK_END);
    lseek(fileb_fd, 0, SEEK_SET);

    fileb_data = malloc(fileb_len);

    if (!fileb_data) {
      close(filea_fd);
      close(fileb_fd);
      free(filea_data);
      filea_data = NULL;
      fprintf(command->out, "cmp: malloc() fail\n");
      return -1;
    }

    read(filea_fd, filea_data, filea_len);
    read(fileb_fd, fileb_data, fileb_len);

    close(filea_fd);
    close(fileb_fd);

    while ((filea_index < filea_len) && (fileb_index < fileb_len)) {

      if (max != 0) {
        if (total >= max) {
          free(fileb_data);
          free(filea_data);

          filea_data = NULL;
          fileb_data = NULL;
          return 0;
        }
      }

      if (filea_data[filea_index] != fileb_data[fileb_index]) {
        fprintf(command->out, "%s %s differ after byte %d", filea, fileb,
                total);

        if (b_flag) {
          fprintf(command->out, " is %.2x %c %.2x %c", filea_data[filea_index],
                  filea_data[filea_index], fileb_data[fileb_index],
                  fileb_data[fileb_index]);
        }

        fprintf((command->out), "\n");

        free(fileb_data);
        free(filea_data);

        filea_data = NULL;
        fileb_data = NULL;
        return 0;
      }

      filea_index++;
      fileb_index++;
      total++;
    }
  }

  free(fileb_data);
  free(filea_data);

  filea_data = NULL;
  fileb_data = NULL;

  return 0;
}

int handle_exit(cmd *command) {
  env *a;
  env *b;

  if (!command) {
    return -1;
  }

  fprintf(command->out, "exiting....\n");

  free_cmd(command);

  a = env_vars;

  while (a) {
    b = a;

    a = a->next;

    free(b->name);
    free(b->value);

    free(b);
  }

  fclose(default_in);
  fclose(default_out);

  exit(0);

  return 0;
}

int handle_export(cmd *command) {
  char *name;
  char *value;
  char *eq;
  env *nv;

  if (command == NULL) {
    return -1;
  }

  if (command->argc == 1) {
    return handle_env(command);
  }

  for (int i = 1; i < command->argc; i++) {
    eq = strchr(command->argv[i], '=');

    if (eq == NULL) {
      continue;
    }

    eq[0] = '\x00';

    name = strdup(command->argv[i]);

    value = strdup(eq + 1);

    nv = getenvvar(name);

    if (nv) {
      setenvvar(name, value);
    } else {
      append_env(makevar(name, value));
    }

    free(name);
    free(value);

    name = NULL;
    value = NULL;
  }

  return 0;
}

char *print_long_ls(char *file, struct stat *st) {
  char perms[11];
  char *user = NULL;
  char *group = NULL;
  struct passwd *pw;
  struct group *g;
  char *modtime = NULL;
  char *final = NULL;

  if (!file || !st) {
    return NULL;
  }

  memset(perms, 0, 11);

  if (st->st_mode & S_ISUID) {
    perms[0] = 's';
  } else if (st->st_mode & S_IFDIR) {
    perms[0] = 'd';
  } else {
    perms[0] = '-';
  }

  perms[1] = ((st->st_mode & S_IRUSR) ? 'r' : '-');
  perms[2] = ((st->st_mode & S_IWUSR) ? 'w' : '-');
  perms[3] = ((st->st_mode & S_IXUSR) ? 'x' : '-');

  perms[4] = ((st->st_mode & S_IRGRP) ? 'r' : '-');
  perms[5] = ((st->st_mode & S_IWGRP) ? 'w' : '-');
  perms[6] = ((st->st_mode & S_IXGRP) ? 'x' : '-');

  perms[7] = ((st->st_mode & S_IROTH) ? 'r' : '-');
  perms[8] = ((st->st_mode & S_IWOTH) ? 'w' : '-');
  perms[9] = ((st->st_mode & S_IXOTH) ? 'x' : '-');

  pw = getpwuid(st->st_uid);

  if (pw == NULL) {
    user = "unknown";
  } else {
    user = pw->pw_name;
  }

  g = getgrgid(st->st_gid);

  if (g == NULL) {
    group = "unknown";
  } else {
    group = g->gr_name;
  }

  modtime = ctime(&st->st_mtim.tv_sec);

  if (!modtime) {
    modtime = "unknown";
  } else {

    modtime[strlen(modtime) - 1] = '\0';
  }

  final = malloc(512);

  if (!final) {
    return NULL;
  }

  snprintf(final, 511, "%s %s %s %s %s", perms, user, group, modtime, file);

  return final;
}

int add_info_link(file_info *fi_root, struct dirent *d, struct stat *st) {
  file_info *nfi;

  if (!fi_root || !d || !st) {
    return 1;
  }

  nfi = malloc(sizeof(file_info));

  if (!nfi) {
    return 1;
  }

  memset(nfi, 0, sizeof(file_info));

  memcpy(&(nfi->d), d, sizeof(struct dirent));
  memcpy(&(nfi->st), st, sizeof(struct stat));

  while (fi_root->next) {
    if (fi_root->next->st.st_mtim.tv_sec > nfi->st.st_mtim.tv_sec) {
      nfi->next = fi_root->next;
      fi_root->next = nfi;
      return 0;
    }

    fi_root = fi_root->next;
  }

  nfi->next = fi_root->next;
  fi_root->next = nfi;

  return 0;
}

int print_sorted_ls(cmd *command, char *file, int l_flag, int a_flag) {
  struct stat st;
  file_info fi_root;
  file_info *walker;
  struct dirent *ent;
  char *long_data = NULL;
  DIR *dir;
  char fullname[PATH_MAX];

  if (!file) {
    return 1;
  }

  memset(&fi_root, 0, sizeof(fi_root));

  dir = opendir(file);

  if (!dir) {
    fprintf(command->out, "ls: cannot access '%s': %s\n", file,
            strerror(errno));
    return 1;
  }

  while ((ent = readdir(dir)) != NULL) {
    if (!a_flag && ent->d_name[0] == '.') {
      continue;
    }

    memset(fullname, 0, PATH_MAX);

    snprintf(fullname, PATH_MAX, "%s/%s", file, ent->d_name);

    if (lstat(fullname, &st)) {
      fprintf(command->out, "ls: cannot stat '%s': %s\n", fullname,
              strerror(errno));
      continue;
    }

    add_info_link(&fi_root, ent, &st);
  }

  closedir(dir);

  walker = fi_root.next;

  while (walker) {
    if (l_flag) {
      long_data = print_long_ls(walker->d.d_name, &(walker->st));

      if (long_data) {
        fprintf(command->out, "%s\n", long_data);
        free(long_data);

        long_data = NULL;
      }
    } else {
      fprintf(command->out, "%s\n", walker->d.d_name);
    }

    walker = walker->next;
  }

  while (fi_root.next) {
    walker = fi_root.next;

    fi_root.next = walker->next;
    free(walker);
  }

  walker = NULL;

  return 0;
}

int print_ls(cmd *command, char *file, int l_flag, int a_flag) {
  DIR *dir = NULL;
  struct dirent *ent = NULL;
  char *longform = NULL;
  struct stat st;
  dir = opendir(file);
  char fullname[PATH_MAX];

  if (!dir) {
    fprintf(command->out, "ls: cannot access '%s': %s\n", file,
            strerror(errno));
    return 1;
  }

  while ((ent = readdir(dir)) != NULL) {
    if (!a_flag && ent->d_name[0] == '.') {
      continue;
    }

    if (l_flag) {
      memset(fullname, 0, PATH_MAX);

      snprintf(fullname, PATH_MAX, "%s/%s", file, ent->d_name);

      if (lstat(fullname, &st)) {
        fprintf(command->out, "ls: cannot stat '%s': %s\n", fullname,
                strerror(errno));
        continue;
      }

      longform = print_long_ls(ent->d_name, &st);

      if (longform) {
        fprintf(command->out, "%s\n", longform);
        free(longform);

        longform = NULL;
      }
    } else {
      fprintf(command->out, "%s\n", ent->d_name);
    }
  }

  closedir(dir);

  return 0;
}

int handle_ls(cmd *command) {

  int l_flag = 0;

  int t_flag = 0;

  int a_flag = 0;

  char *file = NULL;
  int index = 0;
  int c = 0;

  struct stat st;

  if (!command) {
    return -1;
  }

  opterr = 0;
  optind = 0;

  while ((c = getopt(command->argc, command->argv, "alt")) != -1) {
    switch (c) {
    case 'a':
      a_flag = 1;
      break;
    case 'l':
      l_flag = 1;
      break;
    case 't':
      t_flag = 1;
      break;
    case '?':
      if (isprint(optopt))
        fprintf(command->out, "Unknown option `-%c'.\n", optopt);
      else
        fprintf(command->out, "Unknown option character `\\x%x'.\n", optopt);

      return -1;
    default:
      return -1;
    }
  }

  c = 0;
  for (index = optind; index < command->argc; index++) {
    c = 1;
    file = realpath(command->argv[index], NULL);

    if (file == NULL) {
      fprintf(command->out, "ls: cannot access '%s': %s\n",
              command->argv[index], strerror(errno));
    } else {
      memset(&st, 0, sizeof(st));

      if (lstat(file, &st)) {
        continue;
      }

      if (S_ISDIR(st.st_mode)) {
        if (t_flag) {
          print_sorted_ls(command, file, l_flag, a_flag);
        } else {
          print_ls(command, file, l_flag, a_flag);
        }
      } else {
        fprintf(command->out, "%s\n", print_long_ls(command->argv[index], &st));
      }

      free(file);
      file = NULL;
    }
  }

  if (!c) {

    if (lstat(getenvvalue("PWD"), &st)) {
      fprintf(command->out, "ls: cannot access '%s': %s\n", getenvvalue("PWD"),
              strerror(errno));
    }

    if (t_flag) {
      print_sorted_ls(command, getenvvalue("PWD"), l_flag, a_flag);
    } else {
      print_ls(command, getenvvalue("PWD"), l_flag, a_flag);
    }
  }

  return 0;
}

int handle_mkdir(cmd *command) {

  int v_flag = 0;

  char *mode = NULL;

  if (!command) {
    return -1;
  }

  int c = 0;
  int index;

  mode_t mt = S_IRWXU | S_IRGRP | S_IXGRP | S_IROTH | S_IXOTH;

  opterr = 0;
  optind = 0;

  if (command->argc == 1) {
    fprintf(command->out, "mkdir: missing operand\n");
    return -1;
  }

  while ((c = getopt(command->argc, command->argv, "hvm:")) != -1) {
    switch (c) {
    case 'h':
      fprintf(command->out, "mkdir [OPTION] DIR\n\t-h Print this help\n\t-v "
                            "Verbose mode\n\t-m Mode i.e. 777 for rwxrwxrwx\n");
      return 0;
      break;
    case 'v':
      v_flag = 1;
      break;
    case 'm':
      mode = strdup(optarg);
      mt = strtoul(mode, NULL, 8);
      break;
    case '?':
      if (optopt == 'm') {
        fprintf(command->out, "mkdir: option requires an argument -- 'm'\n");
      } else if (isprint(optopt)) {
        fprintf(command->out, "Unknown option `-%c'.\n", optopt);
      } else {
        fprintf(command->out, "Unknown option character `\\x%x'.\n", optopt);
      }

      return -1;
    default:
      return -1;
    }
  }

  droppriv();
  for (index = optind; index < command->argc; index++) {
    if (mkdir(command->argv[index], mt)) {
      fprintf(command->out, "mkdir: cannot create directory ‘%s’: %s\n",
              command->argv[index], strerror(errno));
      continue;
    }

    if (v_flag) {
      fprintf(command->out, "mkdir: created directory '%s'\n",
              command->argv[index]);
    }
  }

  uppriv();

  return 0;
}

int handle_mv(cmd *command) {
  struct stat st;

  if (!command) {
    return -1;
  }

  if (command->argc == 1) {
    fprintf(command->out, "mv: missing file operand\n");
    return -1;
  }

  if (command->argc == 2) {
    fprintf(command->out, "mv: missing destination file operand after '%s'\n",
            command->argv[2]);
    return -1;
  }

  if (lstat(command->argv[1], &st)) {
    fprintf(command->out, "mv: cannot stat '%s': %s\n", command->argv[1],
            strerror(errno));
    return -1;
  }

  if (!can_access_file(command->argv[1], S_IWOTH)) {
    fprintf(command->out, "mv cannot move '%s': permission denied\n",
            command->argv[1]);
    return -1;
  }

  if (rename(command->argv[1], command->argv[2])) {
    fprintf(command->out, "mv: cannot move '%s': %s\n", command->argv[1],
            strerror(errno));
    return -1;
  }

  return 0;
}

int handle_pwd(cmd *command) {
  char pwd[PATH_MAX];

  if (!command) {
    return 1;
  }

  memset(pwd, 0, PATH_MAX);

  fprintf(command->out, "%s\n", getenvvalue("PWD"));

  return 0;
}

int handle_ln(cmd *command) {
  int s_flag = 0;
  char *target_dir = NULL;
  int v_flag = 0;
  char c;
  char *final_destination = NULL;
  char target[PATH_MAX];
  char *fname = NULL;
  struct stat st;
  int index = 0;

  opterr = 0;
  optind = 0;

  if (!command) {
    return -1;
  }

  while ((c = getopt(command->argc, command->argv, "hst:v")) != -1) {
    switch (c) {
    case 's':
      s_flag = 1;
      break;
    case 't':
      target_dir = strdup(optarg);
      break;
    case 'h':
      fprintf(command->out,
              "ln [OPTION] TARGET LINK_NAME\n\t-s make symbolic links instead "
              "of hard links\n\t-t <DIRECTORY> specify the DIRECTORY in which "
              "to create the links\n\t-h Print this help\n\t-v Verbose mode\n");
      return 0;
      break;
    case 'v':
      v_flag = 1;
      break;
    case '?':
      if (optopt == 't') {
        fprintf(command->out, "ln: option requires an argument -- 't'\n");
      } else if (isprint(optopt)) {
        fprintf(command->out, "Unknown option `-%c'.\n", optopt);
      } else {
        fprintf(command->out, "Unknown option character `\\x%x'.\n", optopt);
      }

      if (target_dir) {
        free(target_dir);
        target_dir = NULL;
      }
      return -1;
    default:
      if (target_dir) {
        free(target_dir);
        target_dir = NULL;
      }
      return -1;
    }
  }

  if (command->argc - optind < 2 && target_dir == NULL) {
    fprintf(command->out, "ln: too few arguments\n");

    if (target_dir) {
      free(target_dir);
      target_dir = NULL;
    }
    return -1;
  }

  if (command->argc - optind > 2 && target_dir == NULL) {
    if (stat(command->argv[command->argc - 1], &st)) {
      fprintf(command->out, "ln: cannot access '%s': %s\n",
              command->argv[command->argc - 1], strerror(errno));
      if (target_dir) {
        free(target_dir);

        target_dir = NULL;
      }
      return -1;
    }

    if (S_ISDIR(st.st_mode)) {
      fprintf(command->out, "ln: '%s' is not a directory\n",
              command->argv[command->argc - 1]);
      if (target_dir) {
        free(target_dir);
        target_dir = NULL;
      }
      return -1;
    }

    target_dir = command->argv[command->argc - 1];

    command->argc -= 1;
  }

  if (target_dir) {
    final_destination = realpath(target_dir, NULL);

    if (target_dir) {
      free(target_dir);
      target_dir = NULL;
    }

    if (final_destination == NULL) {
      fprintf(command->out, "ln: cannot access '%s': %s\n", target_dir,
              strerror(errno));
      return -1;
    }

    for (index = optind; index < command->argc; index++) {

      if (stat(command->argv[index], &st)) {
        fprintf(command->out, "ln: cannot access '%s': %s\n",
                command->argv[index], strerror(errno));
        continue;
      }

      fname = basename(command->argv[index]);

      if (!fname) {
        continue;
      }

      memset(target, 0, PATH_MAX);

      snprintf(target, PATH_MAX - 1, "%s/%s", final_destination, fname);

      droppriv();
      if (s_flag) {
        if (symlink(command->argv[index], target)) {
          fprintf(command->out, "ln: failed '%s': %s\n", command->argv[index],
                  strerror(errno));
          uppriv();
          continue;
        }

      } else {
        if (link(command->argv[index], target)) {
          fprintf(command->out, "ln: failed '%s': %s\n", command->argv[index],
                  strerror(errno));
          uppriv();
          continue;
        }
      }
      uppriv();

      if (v_flag) {
        fprintf(command->out, "%s -> %s\n", command->argv[index], target);
      }
    }

    if (final_destination) {
      free(final_destination);
      final_destination = NULL;
    }
  } else {

    droppriv();
    if (s_flag) {
      if (symlink(command->argv[optind], command->argv[optind + 1])) {
        fprintf(command->out, "ln: failed '%s': %s\n", command->argv[optind],
                strerror(errno));
        uppriv();
        return -1;
      }
    } else {
      if (link(command->argv[optind], command->argv[optind + 1])) {
        fprintf(command->out, "ln: failed '%s': %s\n", command->argv[optind],
                strerror(errno));
        uppriv();
        return -1;
      }
    }
    uppriv();

    if (v_flag) {
      fprintf(command->out, "%s -> %s\n", command->argv[index], target);
    }
  }

  return 0;
}

int handle_rev_loop(FILE *fp, cmd *command) {
  char *line = NULL;
  size_t len;
  int bytes_read;
  int nl = 0;

  if (!fp || !command) {
    return -1;
  }

  while ((bytes_read = getline(&line, &len, fp)) > 0) {
    if (!line) {
      return -1;
    }

    if (line[bytes_read - 1] == '\n') {
      bytes_read--;
      nl = 1;
    }

    while (bytes_read - 1 >= 0) {
      fprintf(command->out, "%c", line[bytes_read - 1]);
      bytes_read--;
    }

    if (nl) {
      fprintf(command->out, "\n");
    }

    nl = 0;
  }

  if (line) {
    free(line);
    line = NULL;
  }

  return 0;
}

int handle_rev(cmd *command) {
  int index;
  struct stat st;
  FILE *fp;
  char c;

  if (!command) {
    return -1;
  }

  opterr = 0;
  optind = 0;

  while ((c = getopt(command->argc, command->argv, "h")) != -1) {
    switch (c) {
    case 'h':
      fprintf(command->out, "rev [file1 ...]\n");
      return 0;
      break;
    case '?':
      if (isprint(optopt)) {
        fprintf(command->out, "Unknown option `-%c'.\n", optopt);
      } else {
        fprintf(command->out, "Unknown option character `\\x%x'.\n", optopt);
      }

      return -1;
    default:
      return -1;
    }
  }

  if (!(command->argc - optind)) {
    return handle_rev_loop(curin, command);
  }

  for (index = optind; index < command->argc; index++) {
    if (lstat(command->argv[index], &st)) {
      fprintf(command->out, "rev: '%s': %s\n", command->argv[index],
              strerror(errno));
      continue;
    }

    if (!can_access_file(command->argv[index], S_IROTH)) {
      fprintf(command->out, "rev: '%s': permission denied\n",
              command->argv[index]);
      continue;
    }

    fp = fopen(command->argv[index], "r");

    if (fp == NULL) {
      fprintf(command->out, "rev: '%s': %s\n", command->argv[index],
              strerror(errno));
      continue;
    }

    handle_rev_loop(fp, command);

    fclose(fp);
  }

  return 0;
}

int recursively_remove(const char *fpath, const struct stat *sb, int typeflag,
                       struct FTW *ftwbuf) {
  if (!fpath || !sb || !ftwbuf) {
    return -1;
  }

  if (!can_access_file((char *)fpath, S_IWOTH)) {
    fprintf(rcomm->out, "rm: failed '%s': permission denied\n", fpath);
    return -1;
  }

  int result = remove(fpath);

  if (result) {
    fprintf(rcomm->out, "rm: failed '%s': %s\n", fpath, strerror(errno));
    return -1;
  } else {
    if (rvfl) {
      fprintf(rcomm->out, "removed: %s\n", fpath);
    }
  }

  return result;
}

int handle_rm(cmd *command) {
  int r_flag = 0;
  int v_flag = 0;
  char c;
  int index;

  optind = 0;
  opterr = 0;

  while ((c = getopt(command->argc, command->argv, "rRhv")) != -1) {
    switch (c) {
    case 'r':
    case 'R':
      r_flag = 1;
      break;
    case 'h':
      fprintf(
          command->out,
          "rm [OPTION] FILE \n\t-r -R remove directories and their contents "
          "recursively\n\t-h Print this help\n\t-v Verbose mode\n");
      return 0;
      break;
    case 'v':
      v_flag = 1;
      break;
    case '?':
      if (isprint(optopt)) {
        fprintf(command->out, "Unknown option `-%c'.\n", optopt);
      } else {
        fprintf(command->out, "Unknown option character `\\x%x'.\n", optopt);
      }

      return -1;
    default:
      return -1;
    }
  }

  if (optind - command->argc == 0) {
    fprintf(command->out, "rm: missing operand\n");
    return -1;
  }

  for (index = optind; index < command->argc; index++) {
    if (!r_flag) {
      if (!can_access_file(command->argv[index], S_IWOTH)) {
        fprintf(command->out, "rm: failed '%s': permission denied\n",
                command->argv[index]);
        continue;
      }

      if (remove(command->argv[index])) {
        fprintf(command->out, "rm: failed to remove '%s': %s\n",
                command->argv[index], strerror(errno));
        continue;
      }
    } else {
      rcomm = command;
      rvfl = v_flag;
      nftw(command->argv[index], recursively_remove, 128, FTW_DEPTH | FTW_PHYS);
      rcomm = NULL;
      rvfl = 0;
    }
  }

  return 0;
}

int read_tr_loop(cmd *command, char *setone, char *settwo) {
  char c;
  int retval;
  hit_ctrl_c = 1;
  char *subc;
  int l;

  if (!command) {
    return -1;
  }

  if (!setone) {
    return -1;
  }

  if (settwo) {
    l = strlen(settwo);
  }

  while (hit_ctrl_c) {
    retval = fread(&c, 1, 1, command->in);

    if (retval <= 0) {
      hit_ctrl_c = 0;
      return retval;
    }

    subc = strchr(setone, c);

    if (!subc) {
      fprintf(command->out, "%c", c);
    } else {

      if (settwo == NULL) {
        continue;
      }

      retval = subc - setone;

      if (retval < l) {
        fprintf(command->out, "%c", settwo[retval]);
      } else {
        fprintf(command->out, "%c", settwo[l]);
      }
    }
  }

  return 0;
}

int handle_tr(cmd *command) {
  int c;
  char *setone = NULL;
  char *settwo = NULL;
  int d_flag = 0;
  int set_arg_cnt = 0;

  if (command == NULL) {
    return -1;
  }

  opterr = 0;
  optind = 0;

  while ((c = getopt(command->argc, command->argv, "dh")) != -1) {
    switch (c) {
    case 'd':
      d_flag = 1;
      break;
    case 'h':
      fprintf(command->out, "tr [OPTION] SET1 [SET2]\n\t-d delete characters "
                            "in SET1\n\t-h Print this help\n");
      return 0;
      break;
    case '?':
      if (isprint(optopt)) {
        fprintf(command->out, "Unknown option `-%c'.\n", optopt);
      } else {
        fprintf(command->out, "Unknown option character `\\x%x'.\n", optopt);
      }

      return -1;
    default:
      return -1;
    }
  }

  set_arg_cnt = command->argc - optind;

  if (set_arg_cnt == 0) {
    fprintf(command->out, "tr: missing operand\n");
    return -1;
  } else if (set_arg_cnt == 2 && d_flag) {
    fprintf(command->out, "tr: extra operand %s\n",
            command->argv[command->argc - 1]);
    return -1;
  } else if (set_arg_cnt >= 3) {
    fprintf(command->out, "tr: extra operand %s\n",
            command->argv[command->argc - 1]);
    return -1;
  } else if (set_arg_cnt == 1 && !d_flag) {
    fprintf(command->out, "tr: missing operand after ‘%s’\n",
            command->argv[command->argc]);
    return -1;
  }

  setone = command->argv[optind];

  if (set_arg_cnt == 2) {
    settwo = command->argv[optind + 1];
  }

  read_tr_loop(command, setone, settwo);

  return 0;
}

int handle_unset(cmd *command) {
  env *walker;
  env *t;

  if (command == NULL) {
    return -1;
  }

  for (int i = 1; i < command->argc; i++) {

    if (!getenvvar(command->argv[i])) {
      continue;
    }

    if (!strcmp(command->argv[i], "PWD") || !strcmp(command->argv[i], "USER") ||
        !strcmp(command->argv[i], "HOME")) {
      continue;
    }

    walker = env_vars;

    if (!strcmp(walker->name, command->argv[i])) {
      env_vars = walker->next;

      free(walker->name);
      free(walker->value);

      walker->name = NULL;
      walker->value = NULL;

      free(walker);
      walker = NULL;

      continue;
    }

    while (walker) {
      t = walker->next;

      if (t) {
        if (!strcmp(t->name, command->argv[i])) {
          walker->next = t->next;

          free(t->name);
          free(t->value);

          t->name = NULL;
          t->value = NULL;

          free(t);
          t = NULL;
          break;
        }
      }

      walker = t;
    }
  }

  return 0;
}

char *commands[] = {"cat",  "cd",     "cmp", "cp",    "date",  "echo", "env",
                    "exit", "export", "ln",  "ls",    "mkdir", "mv",   "pwd",
                    "rev",  "rm",     "tr",  "unset", NULL};
void *functions[] = {handle_cat,    handle_cd,    handle_cmp, handle_cp,
                     handle_date,   handle_echo,  handle_env, handle_exit,
                     handle_export, handle_ln,    handle_ls,  handle_mkdir,
                     handle_mv,     handle_pwd,   handle_rev, handle_rm,
                     handle_tr,     handle_unset, NULL};

char *dupbuff(const char *buff, unsigned int start, unsigned int end) {
  char *t;
  int l;

  if (!buff || end <= start) {
    return NULL;
  }

  l = end - start;
  t = malloc(l + 1);

  if (!t) {
    return NULL;
  }

  memset(t, 0, l + 1);
  memcpy(t, buff + start, l);

  return t;
}

int add_cmd_arg(cmd *new_command, char *arg, int length) {
  char **temp;
  char *na;
  int l;

  if (!new_command || !arg) {
    return 0;
  }

  if (new_command->argv == NULL) {
    new_command->argv = malloc(sizeof(char *) * 5);

    if (!new_command->argv) {
      return 0;
    }

    memset(new_command->argv, 0, sizeof(char *) * 5);
    new_command->max_argv = 5;
  }

  if (new_command->argc == new_command->max_argv) {
    l = (new_command->max_argv * 2) * sizeof(char *);

    temp = malloc(l);

    if (!temp) {

      return 0;
    }

    memset(temp, 0, l);

    memcpy(temp, new_command->argv, l / 2);

    free(new_command->argv);

    new_command->argv = temp;
    new_command->max_argv *= 2;
  }

  na = malloc(length + 1);

  if (!na) {

    return 0;
  }

  memset(na, 0, length + 1);

  memcpy(na, arg, length);

  new_command->argv[new_command->argc] = na;
  new_command->argc++;

  return new_command->argc;
}

int valid_command(cmd *c) {
  int i = 0;

  if (!c) {
    return 0;
  }

  if (!c->argv[0]) {
    return 0;
  }

  while (commands[i]) {
    if (!strcmp(commands[i], c->argv[0])) {
      return 1;
    }

    i++;
  }

  return 0;
}

cmd *init_command(char *nc) {
  cmd *new_command;

  if (nc == NULL) {
    return NULL;
  }

  new_command = malloc(sizeof(cmd));

  if (!new_command) {

    return NULL;
  }

  memset(new_command, 0, sizeof(cmd));

  new_command->argv = malloc(sizeof(char *) * 5);

  if (!new_command->argv) {

    return 0;
  }

  memset(new_command->argv, 0, sizeof(char *) * 5);

  new_command->argv[0] = strdup(nc);

  new_command->max_argv = 5;
  new_command->argc = 1;

  new_command->out = curout;
  new_command->in = curin;

  return new_command;
}

char *strrep(char *haystack, char *needle, char *new_needle) {
  char *t = NULL;
  char *ns = NULL;

  t = strstr(haystack, needle);

  if (!t) {
    return haystack;
  }

  ns = malloc(strlen(haystack) + strlen(new_needle) + 1);

  if (!ns) {
    return NULL;
  }

  memset(ns, 0, strlen(haystack) + strlen(new_needle) + 1);

  memcpy(ns, haystack, t - haystack);
  memcpy(ns + (t - haystack), new_needle, strlen(new_needle));
  memcpy(ns + (t - haystack) + strlen(new_needle), t + strlen(needle),
         strlen(t + strlen(needle)));

  return ns;
}

void expand_env_vars(cmd *command) {
  char *val = NULL;
  char *var = NULL;
  char *repd = NULL;

  if (!command) {
    return;
  }

  for (int i = 0; i < command->argc; i++) {
    var = strchr(command->argv[i], '$');

    if (var) {

      val = getenvvalue(var + 1);

      if (val) {
        repd = strrep(command->argv[i], var, val);

        if (repd) {
          free(command->argv[i]);
          command->argv[i] = repd;
        }
      }
    }
  }

  return;
}

int execute_command(cmd *command) {
  int (*cmdFunc)(cmd *);
  int index = 0;
  int retval = 0;

  if (!command) {
    return -1;
  }

  expand_env_vars(command);

  if (!valid_command(command)) {
    fprintf(command->out, "'%s': not a valid command\n", command->argv[0]);
    return -1;
  }

  cmdFunc = NULL;

  while (commands[index]) {
    if (!strcmp(commands[index], command->argv[0])) {
      cmdFunc = functions[index];
      break;
    }
    index++;
  }

  if (cmdFunc) {
    retval = cmdFunc(command);

    last_exit_code = retval;
  }

  return retval;
}

cmd *tokenize_line(char *line, int length) {
  cmd *nc = NULL;
  int state = CMD;
  int index = 0;
  char *start;
  char *end;
  char *tcmd;
  int fildes[2];
  int app = 0;

  if (line == NULL || length == 0) {
    return NULL;
  }

  while (1) {
    switch (state) {
    case CMD:

      start = line + index;
      end = NULL;

      while (index < length) {
        if (line[index] == ' ') {
          state = SPACE;
          end = line + index;
          break;
        } else if (line[index] == ';') {
          state = EXEC_TO_NEWCMD;
          end = line + index;
          break;
        } else if (line[index] == '|') {
          state = PIPE;
          end = line + index;
          break;
        } else if (line[index] == '>') {
          state = ARROW;
          end = line + index;
          break;
        }

        index++;
      }

      if (end == NULL) {
        state = END;

        tcmd = dupbuff(start, 0, length);

        if (!tcmd) {
          state = ERROR;
        }

        nc = init_command(tcmd);

        if (!nc) {
          state = ERROR;
        }

        free(tcmd);
        tcmd = NULL;
      } else {
        tcmd = dupbuff(start, 0, end - start);

        if (!tcmd) {
          state = ERROR;
        }

        nc = init_command(tcmd);

        if (!nc) {
          state = ERROR;
        }

        free(tcmd);
        tcmd = NULL;
      }
      break;
    case EXEC_TO_NEWCMD:

      if (nc) {
        execute_command(nc);
      }

      free_cmd(nc);
      nc = NULL;

      index++;

      state = NEWCMD;
      break;
    case NEWCMD:

      while (line[index] == ' ' && index < length) {
        index++;
      }

      if (index == length) {
        return NULL;
      } else {
        state = CMD;
      }

      break;
    case SPACE:

      index++;

      if (index == length) {
        state = END;
        continue;
      }

      switch (line[index]) {
      case ' ':

        state = SPACE;
        break;
      case ';':
        state = EXEC_TO_NEWCMD;
        break;
      case '-':
        state = ARG;
        break;
      case '|':
        state = PIPE;
        break;
      case '"':
        state = ARG;
        break;
      case '>':
        state = ARROW;
        break;
      default:
        state = ARG;
        break;
      }

      break;
    case ARG:

      start = line + index;
      end = NULL;

      while (index < length) {
        if (line[index] == ' ') {
          state = SPACE;
          end = line + index;
          break;
        } else if (line[index] == ';') {
          state = EXEC_TO_NEWCMD;
          end = line + index;
          break;
        } else if (line[index] == '"') {
          state = OPENQUOTE_ARG;
          break;
        } else if (line[index] == '`') {
          end = line + index;
          state = SUBCMD;
          break;
        } else if (line[index] == '|') {
          state = PIPE;
          end = line + index;
          break;
        } else if (line[index] == '>') {
          state = ARROW;
          end = line + index;
        }

        index++;
      }

      if (state == OPENQUOTE_ARG) {
        continue;
      }

      if (end == NULL) {
        state = END;

        tcmd = dupbuff(start, 0, (line + length) - start);

        if (!tcmd) {
          state = ERROR;
          break;
        }

        if (!add_cmd_arg(nc, tcmd, (line + length) - start)) {
          state = ERROR;
        }

        free(tcmd);
        tcmd = NULL;
      } else {
        tcmd = dupbuff(start, 0, end - start);

        if (!tcmd) {
          state = ERROR;
          break;
        }

        if (!add_cmd_arg(nc, tcmd, end - start)) {
          state = ERROR;
        }

        free(tcmd);
        tcmd = NULL;
      }

      break;
    case ARROW:

      if (index + 1 >= length) {
        index++;
        state = ERROR;
        break;
      }

      if (line[index + 1] == '>') {
        app = 1;
      }

      index += 2;

      if (index >= length) {
        state = ERROR;
        break;
      }

      while (index < length && line[index] == ' ') {
        index++;
      }

      start = line + index;
      end = NULL;

      while (index < length) {
        if (line[index] == '|' || line[index] == '>') {
          state = ERROR;
          break;
        } else if (line[index] == ' ') {
          state = ARG;

          end = line + index;
          break;
        } else if (line[index] == ';') {
          state = NEWCMD;
          end = line + index;
          break;
        }

        index++;
      }

      char *fn = NULL;

      if (end == NULL) {
        fn = strdup(start);
      } else {
        fn = dupbuff(start, 0, end - start);
      }

      if (fn == NULL) {
        fprintf(default_out, "unknown failure\n");
        state = ERROR;
        continue;
      }

      int fd;

      droppriv();
      if (app) {
        fd = open(fn, O_WRONLY | O_APPEND | O_CREAT,
                  S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH);
      } else {
        fd = open(fn, O_WRONLY | O_CREAT | O_TRUNC,
                  S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH);
      }
      uppriv();

      if (fd < 0) {
        fprintf(default_out, "open() '%s' failed: %s\n", fn, strerror(errno));
        state = ERROR;
        free(fn);
        fn = NULL;
        continue;
      }

      free(fn);
      fn = NULL;

      nc->out = fdopen(fd, "w");

      nc->close_out = 1;

      if (nc) {
        execute_command(nc);

        fclose(nc->out);

        free_cmd(nc);

        nc = NULL;
      }

      if (curin_needs_closing) {
        fclose(curin);
      }

      while (index < length && line[index] == ' ') {
        index++;
      }

      if (index >= length) {
        return NULL;
      }

      break;
    case PIPE:

      if (pipe(fildes)) {
        fprintf(default_out, "pipe() failed: %s\n", strerror(errno));
        state = ERROR;
        continue;
      }

      nc->out = fdopen(fildes[1], "w");

      nc->close_out = 1;
      nc->close_in = 1;

      if (nc) {
        execute_command(nc);

        fclose(nc->out);

        free_cmd(nc);
        nc = NULL;
      }

      if (curin_needs_closing) {
        fclose(curin);
      }

      curin = fdopen(fildes[0], "r");
      curin_needs_closing = 1;

      state = NEWCMD;

      index++;
      break;
    case OPENQUOTE_ARG:

      index++;

      start = line + index;
      end = NULL;

      while (index < length && line[index] != '"') {

        index++;
      }

      if (index == length) {
        fprintf(curout, "error: unclosed quote\n");
        state = ERROR;
        break;
      }

      end = line + index;

      tcmd = dupbuff(start, 0, end - start);

      if (!tcmd) {
        state = ERROR;
        break;
      }

      if (!add_cmd_arg(nc, tcmd, end - start)) {
        state = ERROR;
      }

      free(tcmd);
      tcmd = NULL;

      index++;

      while (index < length && line[index] == ' ') {
        index++;
      }

      if (index == length) {
        state = END;
        break;
      }

      switch (line[index]) {
      case ' ':
      case '-':
      case '"':

        state = ARG;
        break;
      case ';':
        state = EXEC_TO_NEWCMD;
        break;
      case '|':
        state = PIPE;
        break;
      case '>':
        state = ARROW;
        break;
      default:
        state = ARG;
        break;
      }

      break;
    case SUBCMD:

      return NULL;
      break;
    case END:

      if (nc) {
        execute_command(nc);

        free_cmd(nc);

        nc = NULL;
      }

      return NULL;
      break;
    case ERROR:

      if (nc) {
        free_cmd(nc);
        nc = NULL;
      }
      return NULL;
    default:

      exit(0);
      break;
    }
  }

  return NULL;
}

int rl(char *l, int m) {
  int i = 0;
  char c = '\x00';

  if (!l) {
    return 0;
  }

  while (i < m) {
    fread(&c, 1, 1, default_in);

    if (c == '\n') {
      return i;
    }

    l[i] = c;
    i++;
  }

  return i;
}

int command_loop() {
  char c[256];
  int i;

  init_env("/home/chess", "/home/chess", "chess");
  curin_needs_closing = 0;
  curout_needs_closing = 0;

  while (1) {
    curin = default_in;
    curout = default_out;

    fprintf(default_out, "%s@%s:%s$ ", user, hostname, getenvvalue("PWD"));

    memset(c, 0, 256);

    i = rl(c, 256);

    tokenize_line(c, i);

    if (curin_needs_closing) {
      fclose(curin);
      curin_needs_closing = 0;
    }

    if (curout_needs_closing) {
      fclose(curout);
      curout_needs_closing = 0;
    }
  }
  return 0;
}

int accept_loop(int fd) {
  int conn_fd = 0;
  struct sockaddr_in ca;

  socklen_t ca_len = sizeof(struct sockaddr_in);
  memset(&ca, 0, sizeof(struct sockaddr_in));

  while (1) {
    conn_fd = accept(fd, (struct sockaddr *)&ca, &ca_len);

    if (conn_fd < 0) {
      fprintf(stderr, "accept() failed: %s\n", strerror(errno));
      close(fd);
      return -1;
    }

    default_in = fdopen(conn_fd, "r");
    default_out = fdopen(conn_fd, "w");

    if (default_in == NULL) {
      fprintf(stderr, "fdopen() in failed: %s\n", strerror(errno));
      exit(-1);
    }

    if (default_out == NULL) {
      fprintf(stderr, "fdopen() out failed: %s\n", strerror(errno));
      exit(-1);
    }

    setvbuf(default_out, NULL, _IONBF, 0);
    setvbuf(default_in, NULL, _IONBF, 0);

    command_loop();
  }

  return 0;
}

int setup_socket(int port) {
  int fd;
  struct sockaddr_in sa;
  int enable = 1;

  fd = socket(AF_INET, SOCK_STREAM, 0);

  if (fd < 0) {
    fprintf(stderr, "socket() failed: %s\n", strerror(errno));
    exit(0);
  }

  memset(&sa, 0, sizeof(struct sockaddr_in));

  sa.sin_family = AF_INET;
  sa.sin_addr.s_addr = INADDR_ANY;
  sa.sin_port = htons(port);

  if (bind(fd, (struct sockaddr *)&sa, sizeof(sa)) < 0) {
    fprintf(stderr, "bind() failed: %s\n", strerror(errno));
    close(fd);
    return -1;
  }

  if (setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &enable, sizeof(enable)) < 0) {
    fprintf(stderr, "setsockopt() failed: %s\n", strerror(errno));
    close(fd);
    return -1;
  }

  if (listen(fd, 0) < 0) {
    fprintf(stderr, "listen() failed: %s\n", strerror(errno));
    close(fd);
    return -1;
  }

  fprintf(stderr, "[INFO] Listener socket on port: %d\n", port);

  return fd;
}

void usage(char *pn) {
  fprintf(stderr, "USAGE: %s -p <port> -s\n", pn);
  fprintf(stderr, "-s Used to specify that the CB will use stdin/stdout.\n");
  fprintf(stderr, "-p Specify the port\n");

  exit(1);
}

int main(int argc, char **argv) {
  int s = 0;
  char c;
  int fd;
  int port;
  char *td = NULL;

  char *t = getenv("PORT");

  if (t != NULL) {
    port = atoi(t);
  } else {
    port = 3004;
  }

  current_egid = getegid();

  while ((c = getopt(argc, argv, "p:s")) != -1)
    switch (c) {
    case 'p':
      port = atoi(optarg);

      break;
    case 's':
      s = 1;
      break;
    case '?':
      if (optopt == 'p') {
        fprintf(stderr, "-%c argument required\n", optopt);
        usage(argv[0]);
      } else {
        fprintf(stderr, "Unknown option\n");
        usage(argv[0]);
      }
    default:
      exit(1);
    }

  signal(SIGINT, sigintHandler);

  fd = open(tokenfile, O_RDONLY);

  if (fd < 0) {
    fprintf(stderr, "Failed to open token file: %s\n", strerror(errno));
    exit(0);
  }

  td = malloc(48);

  if (!td) {
    fprintf(stderr, "Failed to allocate token data: %s\n", strerror(errno));
    close(fd);
    exit(0);
  }

  memset(td, 0, 48);
  strcpy(td, "token: ");

  read(fd, td + 7, 32);
  close(fd);

  td = NULL;

  if (!s) {
    fd = setup_socket(port);

    if (fd < 0) {
      exit(1);
    }

    accept_loop(fd);
  } else {
    default_in = stdin;
    default_out = stdout;

    setvbuf(default_out, NULL, _IONBF, 0);
    setvbuf(default_in, NULL, _IONBF, 0);

    command_loop();
  }
}