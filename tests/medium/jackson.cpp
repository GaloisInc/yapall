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

#include <arpa/inet.h>
#include <bits/stdc++.h>
#include <errno.h>
#include <iostream>
#include <malloc.h>
#include <netdb.h>
#include <netinet/in.h>
#include <stdlib.h>
#include <string>
#include <sys/ioctl.h>
#include <sys/select.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

using namespace std;
class channel {
  string chan_name;
  string history;

public:
  channel(string);
  void add_line(string);
  string get_chan_name(void);
  string get_history(void);
};
channel::channel(string chan_name) {
  this->chan_name = chan_name;
  this->history = "";
  return;
}
static inline void rtrim(std::string &s) {
  s.erase(std::find_if(s.rbegin(), s.rend(),
                       [](int ch) { return !std::isspace(ch); })
              .base(),
          s.end());
}
string channel::get_chan_name(void) { return this->chan_name; }
string channel::get_history(void) { return this->history; }
void channel::add_line(string nl) {
  this->history += nl;
  return;
}
class irc {
  string nick;
  string host;
  string user;
  string server;
  string fullname;
  string ircid;
  string last_msg_target;
  int fd;
  int connected;
  int port;
  int logging;
  vector<channel *> channels;

public:
  irc(int, string);
  irc();
  virtual string get_nick(void);
  virtual void set_nick(string);
  virtual int get_logging(void);
  virtual void set_logging(int);
  virtual string get_host(void);
  virtual void set_host(string);
  virtual int get_fd(void);
  virtual void set_fd(int);
  virtual int get_port(void);
  virtual void set_port(int);
  virtual string get_user(void);
  virtual void set_user(string);
  virtual string get_fullname(void);
  virtual void set_fullname(string);
  virtual string get_server(void);
  virtual void set_server(string);
  virtual string get_ircid(void);
  virtual void set_ircid(string);
  virtual channel *get_channel(string);
  virtual void remove_channel(string);
  virtual int conn(void);
  virtual void prompt(void);
  virtual void print_error(string);
  virtual void print_privs(void);
  virtual void print_chan(vector<string>);
  void handle_nick(vector<string> tokens);
  void handle_msg(vector<string>);
  void handle_disconnect(void);
  void handle_join(vector<string> tokens);
  void handle_ping(void);
  void handle_log(void);
  void handle_pong(void);
  void handle_dns(void);
  void parse_server_msg(string);
  string readline(int);
  void writeline(string, int);
  void list_channels(void);
  void add_history_line(string channel, string message);
  void handle_join_response(vector<string>);
  void handle_namereply(vector<string>);
  void handle_endofnames(vector<string>);
  void handle_privmsg_response(vector<string>);
  void handle_welcome_msg(vector<string>);
  void handle_notice(vector<string>);
  void handle_luserlist(vector<string>);
  void handle_lchanlist(vector<string>);
  void handle_luserme(vector<string>);
  void handle_ircid(vector<string>);
  void handle_localusers(vector<string>);
  void handle_invite(vector<string>);
  void handle_globalusers(vector<string>);
  void handle_motdstart(vector<string>);
  void handle_repltopicset(vector<string>);
  void handle_motd(vector<string>);
  void handle_list();
  void handle_topic(vector<string>);
  void handle_newtopic(vector<string>);
  void handle_replnotopic(vector<string>);
  void handle_endmotd(vector<string>);
  void handle_nicknameinuse(vector<string>);
  void handle_part(vector<string>);
  void handle_replpass(vector<string>);
  void handle_replwho(vector<string>);
  void handle_part_response(vector<string>);
  void handle_repluserhost(vector<string>);
  void handle_replnoperm(vector<string>);
  void handle_replinfo(vector<string>);
  void handle_watch(vector<string>);
  void handle_replnotonchan(vector<string>);
  void handle_userhost(vector<string>);
  void handle_replstatscmds(vector<string>);
  void handle_repllist(vector<string>);
  void handle_endreplinfo(vector<string>);
  void handle_statsdline(vector<string>);
  void handle_luserop(vector<string>);
  void handle_lparams(vector<string>);
  void handle_repladmin(vector<string>);
  void handle_replstatslinkinfo(vector<string>);
  void handle_replstatsuptime(vector<string>);
  void handle_plzwait(vector<string>);
  void handle_replendlist(vector<string>);
  void handle_time();
  void handle_luserunknown(vector<string>);
  void handle_replquit(vector<string>);
  void handle_admin(vector<string>);
  void handle_repladminloc1(vector<string>);
  void handle_replendofstats(vector<string>);
  void handle_names(vector<string>);
  void handle_repladminloc2(vector<string>);
  void handle_replstatuline(vector<string>);
  void handle_away(vector<string>);
  void handle_newnick(vector<string>);
  void handle_userip(vector<string>);
  void handle_unkcmd(vector<string>);
  void handle_repladminemail(vector<string>);
  void handle_nosuchchannel(vector<string>);
  void handle_nosuchnick(vector<string>);
  void handle_ison(vector<string>);
  void handle_stats(vector<string>);
  void handle_replison(vector<string>);
  void handle_repltopic(vector<string>);
  void handle_replendofwho(vector<string>);
  void handle_replnowaway(vector<string>);
  void handle_replunaway(vector<string>);
  void handle_inviting(vector<string>);
  void handle_replaway(vector<string>);
  void handle_cmdmotd(vector<string>);
  void handle_repltime(vector<string>);
  void handle_invite_response(vector<string>);
  void handle_who(vector<string>);
  void handle_info();
};
void irc::print_privs(void) {
  cout << "Nick: " << this->nick << endl;
  cout << "Host: " << this->host << endl;
  cout << "fd: " << this->fd << endl;
  cout << "port: " << this->port << endl;
  return;
}
string irc::readline(int fd) {
  char *line = (char *)malloc(1024);
  string ns;
  int i = 0;
  char c = '\0';
  if (!line) {
    this->print_error("malloc() error");
    return "";
  }
  memset(line, 0, 1024);
  while (i < 1024 && c != '\n') {
    if (read(fd, &c, 1) < 0) {
      this->print_error("Failed to read data");
      free(line);
      return "";
    }
    if (c == '\n') {
      continue;
    }
    line[i] = c;
    i++;
  }
  ns = string(line);
  free(line);
  return ns;
}
void irc::writeline(string line, int fd) {
  if (write(fd, line.c_str(), line.size()) < 0) {
    this->print_error("Failed to write: " + string(strerror(errno)));
  }
  return;
}
void irc::handle_repltime(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid server message: 138519");
    return;
  }
  for (int i = 3; i < tokens.size(); i++) {
    if (tokens[i][0] == ':') {
      tokens[i] = tokens[i].substr(1, tokens[i].length() - 1);
    }
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
string irc::get_host(void) { return this->host; }
string irc::get_nick(void) { return this->nick; }
void irc::set_host(string host) { this->host = host; }
void irc::set_nick(string nick) { this->nick = nick; }
int irc::get_fd(void) { return this->fd; }
void irc::set_fd(int fd) { this->fd = fd; }
int irc::get_logging(void) { return this->logging; }
void irc::set_logging(int logging) { this->logging = logging; }
string irc::get_server(void) { return this->server; }
void irc::set_server(string server) { this->server = server; }
int irc::get_port(void) { return this->port; }
void irc::set_port(int port) { this->port = port; }
string irc::get_user(void) { return this->user; }
void irc::set_user(string user) { this->user = user; }
string irc::get_fullname(void) { return this->fullname; }
void irc::set_fullname(string fullname) { this->fullname = fullname; }
string irc::get_ircid(void) { return this->ircid; }
void irc::set_ircid(string ircid) { this->ircid = ircid; }
void irc::print_error(string err) {
  string line = "";
  line += " -!- jackson: ";
  line += err;
  this->add_history_line("main", line);
  return;
}
irc::irc(int port, string host) {
  char *user = NULL;
  channel *nc = NULL;
  this->port = port;
  this->host = host;
  this->connected = 0;
  user = getenv("IRCUSER");
  if (!user) {
    user = (char *)"chessuser";
  }
  this->nick = string(user);
  this->user = string(user);
  this->fullname = "anonymous";
  this->last_msg_target = "";
  this->logging = 0;
  nc = new channel("main");
  if (nc) {
    this->channels.push_back(nc);
  }
}
irc::irc() {
  char *host = getenv("HOST");
  char *port = getenv("PORT");
  char *user = getenv("IRCUSER");
  char *nick = getenv("IRCNICK");
  channel *nc = NULL;
  if (!user) {
    user = (char *)"jackson";
  }
  if (!nick) {
    nick = (char *)"chess";
  }
  if (!host) {
    host = (char *)"localhost";
  }
  if (!host || !port) {
    cout << "[ERROR] Failed to get the PORT" << endl;
    exit(1);
  }
  this->host = string(host);
  try {
    this->port = stoi(port);
  } catch (invalid_argument const &e) {
    cout << "[ERROR] invalid argument: " << port << endl;
    exit(1);
  } catch (out_of_range const &e) {
    cout << "[ERROR] out of range: " << port << endl;
    exit(1);
  }
  this->nick = string(nick);
  this->user = string(user);
  this->fullname = "anonymous";
  this->last_msg_target = "";
  this->connected = 0;
  this->logging = 0;
  nc = new channel("main");
  if (nc) {
    this->channels.push_back(nc);
  }
}
int irc::conn(void) {
  struct sockaddr_in serv_addr;
  int fd;
  string line;
  struct hostent *ht = NULL;
  if (this->connected) {
    this->print_error("Already connected");
    return -1;
  }
  ht = gethostbyname(this->get_server().c_str());
  if (!ht) {
    this->print_error("Failed to get address: '" + this->get_server() +
                      "': " + string(strerror(errno)));
    return -1;
  }
  if ((fd = socket(AF_INET, SOCK_STREAM, 0)) < 0) {
    this->print_error("Socket creation error");
    return -1;
  }
  memset(&serv_addr, '0', sizeof(serv_addr));
  serv_addr.sin_family = AF_INET;
  serv_addr.sin_port = htons(this->get_port());
  if (inet_pton(AF_INET, inet_ntoa(*((struct in_addr *)(ht->h_addr))),
                &serv_addr.sin_addr) <= 0) {
    this->print_error("Invalid address: " +
                      string(inet_ntoa(*((struct in_addr *)(ht->h_addr)))));
    return -1;
  }
  if (connect(fd, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) < 0) {
    this->print_error("Failed to connect: " + this->get_server() + ":" +
                      to_string(this->get_port()) + " : " +
                      string(strerror(errno)));
    return -1;
  }
  this->connected = 1;
  this->fd = fd;
  this->writeline("NICK " + this->get_nick() + "\n", fd);
  this->writeline(
      "USER " + this->get_user() + " 0 * :" + this->get_fullname() + "\n", fd);
  cout << "Connected..." << endl;
  return 0;
}
void irc::print_chan(vector<string> tokens) {
  channel *c = NULL;
  if (tokens.size() < 2) {
    this->print_error("requires an argument");
    return;
  }
  c = this->get_channel(tokens[1]);
  if (c == NULL) {
    this->print_error("invalid argument");
    return;
  }
  cout << "[ " << tokens[1] << " ] " << endl;
  cout << c->get_history() << endl;
  return;
}
void irc::handle_nick(vector<string> tokens) {
  string new_nick = "";
  string cmd = "";
  if (tokens.size() != 2) {
    this->print_error("/nick <new-nick>");
    return;
  }
  new_nick = tokens[1];
  cmd = "NICK " + new_nick + "\n";
  this->writeline(cmd, this->fd);
  return;
}
void irc::handle_msg(vector<string> tokens) {
  string targets;
  vector<string> tgt_v;
  string temp;
  string message = "";
  channel *nc = NULL;
  if (!this->connected) {
    this->print_error("not connected");
    return;
  }
  if (tokens.size() < 3) {
    this->print_error("/msg missing parameters");
    return;
  }
  targets = tokens[1];
  stringstream ss(targets);
  while (getline(ss, temp, ',')) {
    tgt_v.push_back(temp);
  }
  for (int i = 2; i < tokens.size(); i++) {
    message += tokens[i] + " ";
  }
  for (int i = 0; i < tgt_v.size(); i++) {
    if (tgt_v[i] == "*") {
      tgt_v[i] = last_msg_target;
    }
    this->writeline("PRIVMSG " + tgt_v[i] + " : " + message + "\n", this->fd);
    this->last_msg_target = tgt_v[i];
    if (this->get_channel(tgt_v[i]) == NULL) {
      nc = new channel(tgt_v[i]);
      if (nc) {
        this->channels.push_back(nc);
      }
    }
    this->add_history_line(tgt_v[i], "< " + this->get_nick() + "> " + message);
  }
  return;
}
void irc::handle_disconnect(void) {
  if (!this->connected) {
    this->print_error("Not connected");
    return;
  }
  this->writeline("QUIT\n", this->fd);
  this->connected = 0;
  close(this->fd);
  for (auto i = this->channels.begin(); i != this->channels.end(); ++i) {
    delete *i;
  }
  this->channels.clear();
  return;
}
channel *irc::get_channel(string chan) {
  for (int i = 0; i < this->channels.size(); i++) {
    if (this->channels[i]->get_chan_name() == chan) {
      return this->channels[i];
    }
  }
  return NULL;
}
void irc::remove_channel(string chan) {
  for (int i = 0; i < this->channels.size(); i++) {
    if (this->channels[i]->get_chan_name() == chan) {
      this->channels.erase(this->channels.begin() + i);
      return;
    }
  }
  return;
}
void irc::list_channels(void) {
  for (int i = 0; i < this->channels.size(); i++) {
    cout << i << ") " << this->channels[i]->get_chan_name() << endl;
  }
  return;
}
void irc::handle_join(vector<string> tokens) {
  string targets;
  vector<string> tgt_v;
  string temp;
  channel *nc;
  if (!this->connected) {
    this->print_error("Not connected");
    return;
  }
  if (tokens.size() < 2) {
    this->print_error("/join missing parameters");
    return;
  }
  targets = tokens[1];
  stringstream ss(targets);
  while (getline(ss, temp, ',')) {
    tgt_v.push_back(temp);
  }
  for (int i = 0; i < tgt_v.size(); i++) {
    if (tgt_v[i][0] != '#') {
      tgt_v[i] = "#" + tgt_v[i];
    }
    this->writeline("JOIN " + tgt_v[i] + "\n", this->fd);
    nc = this->get_channel(tgt_v[i]);
    if (nc == NULL) {
      nc = new channel(tgt_v[i]);
      if (nc) {
        this->channels.push_back(nc);
      }
    }
  }
  return;
}
void irc::handle_invite(vector<string> tokens) {
  string invitee = "";
  string chan = "";
  if (tokens.size() < 3) {
    this->print_error("Not enough parameters given");
    return;
  }
  invitee = tokens[1];
  chan = tokens[2];
  this->writeline("INVITE " + invitee + " " + chan + "\n", this->fd);
  return;
}
void irc::handle_replinfo(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid message format: 98798745433");
    return;
  }
  tokens[3] = tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_endreplinfo(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid message format: 234y54");
    return;
  }
  tokens[3] = tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
  return;
}
void irc::handle_ping() {
  if (!this->connected) {
    this->print_error("Not connected");
    return;
  }
  this->writeline("PING " + this->get_server() + "\n", this->fd);
  return;
}
void irc::handle_pong() {
  if (!this->connected) {
    this->print_error("Not connected");
    return;
  }
  this->writeline("PONG " + this->get_server() + "\n", this->fd);
  return;
}
void irc::add_history_line(string chan, string message) {
  channel *nc = NULL;
  time_t s;
  struct tm *current_time;
  string command = "";
  string arg = "";
  s = time(NULL);
  current_time = localtime(&s);
  for (int i = 0; i < this->channels.size(); i++) {
    if (this->channels[i]->get_chan_name() == chan) {
      nc = this->channels[i];
    }
  }
  if (nc == NULL) {
    cout << "Received a message from an unjoined channel: " << chan << endl;
    return;
  }
  string logline = to_string(current_time->tm_hour) + ":" +
                   to_string(current_time->tm_min) + " " + message;
  nc->add_line(logline + "\n");
  std::cout << logline << endl;
  if (this->logging) {
    command = string(getenv("SHELL"));
    if (chan[0] == '#') {
      chan = "\\" + chan;
    }
    arg = "echo \\\"" + logline + "\\\" >> " + chan + ".log";
    string a = "-c";
    string finalcmd = command + " ";
    FILE *f = NULL;
    finalcmd += a + " ";
    finalcmd += "\"";
    finalcmd += arg + "\"";
    f = popen(finalcmd.c_str(), "r");
    fclose(f);
  }
  return;
}
string parse_to_get_name(string s) {
  string name = "";
  size_t pos = string::npos;
  if (s[0] == ':') {
    s = s.substr(1, s.length() - 1);
  }
  pos = s.find_first_of('!');
  if (pos == string::npos) {
    return "";
  }
  name = s.substr(0, pos);
  return name;
}
string parse_to_get_host(string s) {
  string host = "";
  size_t pos = string::npos;
  if (s[0] == ':') {
    s = s.substr(1, s.length() - 1);
  }
  pos = s.find_first_of('!');
  if (pos == string::npos) {
    return "";
  }
  host = s.substr(pos + 1, s.length() - 1);
  return host;
}
void irc::handle_join_response(vector<string> tokens) {
  string joiner = "";
  string host = "";
  string room = "";
  if (tokens.size() != 3) {
    this->print_error("invalid join response from server");
    return;
  }
  joiner = parse_to_get_name(tokens[0]);
  if (joiner == "") {
    this->print_error("failed to parse value from string");
    return;
  }
  host = parse_to_get_host(tokens[0]);
  if (host == "") {
    this->print_error("failed to parse value from string");
    return;
  }
  room = tokens[2];
  if (room[0] == ':') {
    room = room.substr(1, room.length() - 1);
  }
  rtrim(room);
  this->add_history_line(room, "-!- " + joiner + " [" + host + "] has joined " +
                                   room);
  return;
}
void irc::handle_namereply(vector<string> tokens) {
  string chan = "";
  string nick = "";
  channel *c = NULL;
  string data = "";
  if (tokens.size() < 6) {
    this->print_error("invalid response from server: 2l35ih");
    return;
  }
  chan = tokens[4];
  c = this->get_channel(chan);
  if (c == NULL) {
    this->print_error("unknown channel: " + chan);
    return;
  }
  data = "[Users " + chan + "]";
  this->add_history_line(chan, data);
  data = "";
  for (int i = 5; i < tokens.size(); i++) {
    if (tokens[i][0] == ':') {
      tokens[i] = tokens[i].substr(1, tokens[i].length() - 1);
    }
    data += "[" + tokens[i] + "] ";
  }
  this->add_history_line(chan, data);
  this->add_history_line(chan, "-!- Chess: " + chan + ": Total of " +
                                   to_string(tokens.size() - 5) + " nicks");
  return;
}
void irc::handle_endofnames(vector<string> tokens) { return; }
void irc::handle_privmsg_response(vector<string> tokens) {
  string name = "";
  string host = "";
  string chan = "";
  string msg = "";
  channel *nc = NULL;
  if (tokens.size() < 4) {
    this->print_error("invalid server message: 2");
    return;
  }
  name = parse_to_get_name(tokens[0]);
  host = parse_to_get_host(tokens[0]);
  if (name == "" || host == "") {
    this->print_error("invalid server message: 3");
    return;
  }
  chan = tokens[2];
  for (int i = 3; i < tokens.size(); i++) {
    if (tokens[i][0] == ':') {
      tokens[i] = tokens[i].substr(1, tokens[i].length() - 1);
    }
    msg += tokens[i] + " ";
  }
  if (chan == this->get_nick()) {
    chan = name;
  }
  if (this->get_channel(chan) == NULL) {
    nc = new channel(chan);
    if (nc) {
      this->channels.push_back(nc);
    }
  }
  this->add_history_line(chan, "< " + name + "> " + msg);
  return;
}
void irc::handle_notice(vector<string> tokens) { return; }
void irc::handle_ircid(vector<string> tokens) {
  if (tokens.size() != 7) {
    this->print_error("invalid server message: 5");
    return;
  }
  this->add_history_line("main", tokens[3] + " your unique ID");
  this->set_ircid(tokens[3]);
  return;
}
void irc::handle_welcome_msg(vector<string> tokens) { return; }
void irc::handle_luserlist(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() != 13) {
    this->print_error("invalid message format: 3");
    return;
  }
  tokens[3] = tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_lchanlist(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() != 6) {
    this->print_error("message error: 4");
    return;
  }
  data += tokens[3] + " channels formed";
  this->add_history_line("main", data);
  return;
}
void irc::handle_luserme(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid message format: 6");
    return;
  }
  tokens[3] = tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_localusers(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() != 11) {
    this->print_error("invalid message format: 7");
    return;
  }
  tokens[5] = tokens[5].substr(1, tokens[5].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_luserop(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() != 7) {
    this->print_error("invalid server message: 14");
    return;
  }
  tokens[4] = tokens[4].substr(1, tokens[4].length() - 1);
  data += tokens[3] + " ";
  for (int i = 4; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_luserunknown(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 6) {
    this->print_error("invalid server message: 15");
    return;
  }
  tokens[4] = tokens[4].substr(1, tokens[4].length() - 1);
  data += tokens[3] + " ";
  for (int i = 4; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_globalusers(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() != 11) {
    this->print_error("invalid message format: 8");
    return;
  }
  tokens[5] = tokens[5].substr(1, tokens[5].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_motdstart(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid message format: 9");
    return;
  }
  tokens[3] = tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_motd(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid message format: 10");
    return;
  }
  tokens[3] = tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_endmotd(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid message format: 11");
    return;
  }
  tokens[3] = tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_part(vector<string> tokens) {
  string targets;
  vector<string> tgt_v;
  string temp;
  string message = "";
  if (!this->connected) {
    this->print_error("not connected");
    return;
  }
  if (tokens.size() < 2) {
    this->print_error("/part missing parameters");
    return;
  }
  targets = tokens[1];
  stringstream ss(targets);
  while (getline(ss, temp, ',')) {
    tgt_v.push_back(temp);
  }
  for (int i = 2; i < tokens.size(); i++) {
    message += tokens[i] + " ";
  }
  for (int i = 0; i < tgt_v.size(); i++) {
    if (tgt_v[i] == "*") {
      tgt_v[i] = last_msg_target;
    }
    this->writeline("PART " + tgt_v[i] + " : " + message + "\n", this->fd);
    this->last_msg_target = tgt_v[i];
    if (this->get_channel(tgt_v[i]) == NULL) {
      this->remove_channel(tgt_v[i]);
    }
  }
  return;
}
void irc::handle_nicknameinuse(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 5) {
    this->print_error("invalid message format: 12");
    return;
  }
  data += "Nick " + tokens[3] + " is already in use.";
  this->set_nick(tokens[4]);
  this->add_history_line("main", data);
  return;
}
void irc::handle_info() {
  string req = "INFO";
  this->writeline(req + "\n", this->fd);
  return;
}
void irc::handle_ison(vector<string> tokens) {
  string req = "ISON ";
  if (tokens.size() < 2) {
    this->print_error("-!- jackson Not enough parameters given");
    return;
  }
  for (int i = 1; i < tokens.size(); i++) {
    req += tokens[i] + " ";
  }
  this->writeline(req + "\n", this->fd);
  return;
}
void irc::handle_replison(vector<string> tokens) {
  string data = "-!- Users online: ";
  if (tokens.size() < 4) {
    this->print_error("server message error: 948181");
    return;
  }
  tokens[3] = tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_statsdline(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid message format: 11");
    return;
  }
  tokens[3] = tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_part_response(vector<string> tokens) {
  string data = "-!- ";
  string chan = "";
  string nick = "";
  if (tokens.size() < 4) {
    this->print_error("invalid server message: laskdjifn");
    return;
  }
  chan = tokens[2];
  nick = parse_to_get_name(tokens[0]);
  data += nick + " left: \"";
  tokens[3] = tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  data += "\"";
  if (nick == this->nick) {
    this->add_history_line("main", data);
    return;
  }
  this->add_history_line(chan, data);
  return;
}
void irc::handle_who(vector<string> tokens) {
  string req = "WHO";
  if (tokens.size() > 1) {
    req += " " + tokens[1];
  } else {
    req += " *";
  }
  this->writeline(req + "\n", this->fd);
  return;
}
void irc::handle_lparams(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid message format: 11");
    return;
  }
  for (int i = 3; i < tokens.size(); i++) {
    if (tokens[i][0] == ':') {
      tokens[i] = tokens[i].substr(1, tokens[i].length() - 1);
    }
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_plzwait(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid message format: 11");
    return;
  }
  tokens[3] = tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_admin(vector<string> tokens) {
  string req = "ADMIN";
  if (tokens.size() > 1) {
    req += " " + tokens[1];
  }
  this->writeline(req + "\n", this->fd);
  return;
}
void irc::handle_cmdmotd(vector<string> tokens) {
  string req = "MOTD";
  if (tokens.size() > 1) {
    req += " " + tokens[1];
  }
  this->writeline(req + "\n", this->fd);
  return;
}
void irc::handle_away(vector<string> tokens) {
  string req = "AWAY";
  if (tokens.size() > 1) {
    req += " " + tokens[1];
  }
  this->writeline(req + "\n", this->fd);
  return;
}
void irc::handle_watch(vector<string> tokens) {
  string req = "";
  if (this->connected == 0) {
    this->print_error("not connected");
    return;
  }
  if (tokens.size() < 2) {
    this->print_error("too few arguments");
    return;
  }
  for (int i = 1; i < tokens.size(); i++) {
    req = "WATCH ";
    if (tokens[i][0] != '-' && tokens[i][0] != '+') {
      tokens[i] = "+" + tokens[i];
    }
    req += tokens[i];
    this->writeline(req + "\n", this->fd);
  }
  return;
}
void irc::handle_userip(vector<string> tokens) {
  string req = "";
  if (this->connected == 0) {
    this->print_error("not connected");
    return;
  }
  if (tokens.size() < 2) {
    this->print_error("too few arguments");
    return;
  }
  for (int i = 1; i < tokens.size(); i++) {
    req = "USERIP " + tokens[i];
    this->writeline(req + "\n", this->fd);
  }
  return;
}
void irc::handle_userhost(vector<string> tokens) {
  string req = "";
  if (this->connected == 0) {
    this->print_error("not connected");
    return;
  }
  if (tokens.size() < 2) {
    this->print_error("too few arguments");
    return;
  }
  for (int i = 1; i < tokens.size(); i++) {
    req = "USERHOST " + tokens[i];
    this->writeline(req + "\n", this->fd);
  }
  return;
}
void irc::handle_inviting(vector<string> tokens) {
  string invitee = "";
  string room = "";
  if (tokens.size() != 5) {
    this->print_error("invalid server string: 234");
    return;
  }
  invitee = tokens[3];
  room = tokens[4];
  this->add_history_line("main", "-!- Inviting " + invitee + " to " + room);
  return;
}
void irc::handle_replnoperm(vector<string> tokens) {
  string data = "-!-";
  if (tokens.size() < 4) {
    this->print_error("invalid server string: nhjuy783");
    return;
  }
  for (int i = 3; i < tokens.size(); i++) {
    if (tokens[i][0] == ':') {
      tokens[i] = tokens[i].substr(1, tokens[i].length() - 1);
    }
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_nosuchnick(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 5) {
    this->print_error("invalid server message: 23453");
    return;
  }
  data += tokens[3];
  for (int i = 4; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_nosuchchannel(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 5) {
    this->print_error("invalid server message: 3e12d3c");
    return;
  }
  data += tokens[3];
  for (int i = 4; i < tokens.size(); i++) {
    data += tokens[i] + " ";
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_time() {
  string req = "TIME";
  if (!this->connected) {
    this->print_error("not connected");
    return;
  }
  this->writeline(req + "\n", this->fd);
  return;
}
void irc::handle_names(vector<string> tokens) {
  string req = "";
  if (this->connected == 0) {
    this->print_error("not connected");
    return;
  }
  req = "NAMES";
  for (int i = 1; i < tokens.size(); i++) {
    req += " " + tokens[i];
  }
  this->writeline(req + "\n", this->fd);
  return;
}
void irc::handle_list() {
  string req = "LIST";
  if (!this->connected) {
    this->print_error("not connected");
    return;
  }
  this->writeline(req + "\n", this->fd);
  return;
}
void irc::handle_invite_response(vector<string> tokens) {
  string nick = "";
  string room = "";
  if (tokens.size() < 4) {
    this->print_error("invalid server message: 94828");
    return;
  }
  nick = parse_to_get_name(tokens[0]);
  room = tokens[3].substr(1, tokens[3].length() - 1);
  this->add_history_line("main", "-!- " + nick + " invites you to " + room);
  return;
}
void irc::handle_unkcmd(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 5) {
    this->print_error("server message error: 6466732");
    return;
  }
  data += tokens[3] + ": ";
  data += tokens[4].substr(1, tokens[4].length() - 1);
  for (int i = 5; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_log() {
  this->set_logging(1);
  this->add_history_line("main", "-!- Set logging");
  return;
}
void irc::handle_topic(vector<string> tokens) {
  string req = "TOPIC ";
  if (tokens.size() < 2) {
    this->print_error("-!- jackson Not enough parameters given");
    return;
  }
  req += tokens[1];
  for (int i = 2; i < tokens.size(); i++) {
    req += " " + tokens[i];
  }
  this->writeline(req + "\n", this->fd);
  return;
}
void irc::handle_replquit(vector<string> tokens) {
  string data = "-!- ";
  string nick = "";
  if (tokens.size() < 3) {
    this->print_error("invalid server message: 23k3k23k335");
    return;
  }
  nick = parse_to_get_name(tokens[0]);
  data += nick + " quit: ";
  data += tokens[2].substr(1, tokens[2].length() - 1);
  for (int i = 3; i < tokens.size(); i++) {
    data += tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_newnick(vector<string> tokens) {
  string oldnick = "";
  string newnick = "";
  string data = "";
  if (tokens.size() < 3) {
    this->print_error("invalid server message: dk3kh33i");
    return;
  }
  oldnick = parse_to_get_name(tokens[0]);
  newnick = tokens[2].substr(1, tokens[2].length() - 1);
  if (oldnick == this->get_nick()) {
    this->set_nick(newnick);
    data += "-!- You're now known as " + newnick;
  } else {
    data += "-!- " + oldnick + " is now known as " + newnick;
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_newtopic(vector<string> tokens) {
  string chan = "";
  string nick = "";
  string data = "";
  if (tokens.size() < 4) {
    this->print_error("invalid server message: k3k3h2o4uh2");
    return;
  }
  chan = tokens[2];
  nick = parse_to_get_name(tokens[0]);
  data += nick + " changed topic to: ";
  data += tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 4; i < tokens.size(); i++) {
    data += tokens[i];
  }
  this->add_history_line(chan, data);
  return;
}
void irc::handle_replnotonchan(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("server message error: 334j324h5");
    return;
  }
  data += tokens[3] + ": ";
  data += tokens[4].substr(1, tokens[4].length() - 1);
  for (int i = 5; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_repltopicset(vector<string> tokens) {
  string data = "Topic set to: ";
  string chan = "";
  if (tokens.size() < 4) {
    this->print_error("server message error: kfeogij58");
    return;
  }
  chan = tokens[2];
  data = chan + " topic set to: ";
  data += tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 4; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line(chan, data);
  return;
}
void irc::handle_repltopic(vector<string> tokens) {
  string data = "";
  string chan = "";
  if (tokens.size() < 5) {
    this->print_error("server message error: 92485ny434");
    return;
  }
  chan = tokens[3];
  data = chan + " topic set to: ";
  data += tokens[4].substr(1, tokens[4].length() - 1);
  for (int i = 5; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line(chan, data);
  return;
}
void irc::handle_replendofwho(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 5) {
    this->print_error("server message error: asdj4j3k");
    return;
  }
  data += tokens[5].substr(1, tokens[4].length() - 1);
  for (int i = 6; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_stats(vector<string> tokens) {
  string req = "STATS ";
  if (tokens.size() != 2) {
    this->print_error("invalid number of parameters");
    return;
  }
  if (tokens[1] == "serv_list") {
    req += "l";
  } else if (tokens[1] == "cmd_cnt") {
    req += "m";
  } else if (tokens[1] == "op_list") {
    req += "o";
  } else if (tokens[1] == "up_time") {
    req += "u";
  } else if (tokens[1] == "resource") {
    req += "r";
  } else if (tokens[1] == "gen_stat") {
    req += "t";
  } else if (tokens[1] == "mem") {
    req += "z";
  } else {
    this->print_error("unknown option: " + tokens[1]);
    return;
  }
  this->writeline(req + "\n", this->fd);
  return;
}
void irc::handle_replnowaway(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("server message error: 4472948");
    return;
  }
  data += tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 4; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_replunaway(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("server message error: 238223923");
    return;
  }
  data += tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 4; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_replaway(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 5) {
    this->print_error("server message error: 592838");
    return;
  }
  data += tokens[3] + " is away: ";
  data += tokens[4].substr(1, tokens[4].length() - 1);
  for (int i = 5; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_repladmin(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid server message: 173461");
    return;
  }
  data += tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 4; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_repladminloc1(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid server message: 7372937");
    return;
  }
  data += tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 4; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_repladminloc2(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid server message: 394812");
    return;
  }
  data += tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 4; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_replpass(vector<string> tokens) { return; }
void irc::handle_repladminemail(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid server message: 83873");
    return;
  }
  data += tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 4; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_replstatslinkinfo(vector<string> tokens) {
  string data = "-!-";
  if (tokens.size() < 10) {
    this->print_error("invalid server message: 3kh23092u3hi");
    return;
  }
  for (int i = 3; i < tokens.size(); i++) {
    if (tokens[i][0] == ':') {
      tokens[i] = tokens[i].substr(1, tokens[i].length() - 1);
    }
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_replendofstats(vector<string> tokens) {
  string data = "-!-";
  if (tokens.size() < 5) {
    this->print_error("invalid server message: 8we9uoisdhfjl");
    return;
  }
  for (int i = 3; i < tokens.size(); i++) {
    if (tokens[i][0] == ':') {
      tokens[i] = tokens[i].substr(1, tokens[i].length() - 1);
    }
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_replstatscmds(vector<string> tokens) {
  string data = "-!-";
  if (tokens.size() < 5) {
    this->print_error("invalid server message: sd98foikjlm");
    return;
  }
  for (int i = 3; i < tokens.size(); i++) {
    if (tokens[i][0] == ':') {
      tokens[i] = tokens[i].substr(1, tokens[i].length() - 1);
    }
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_repluserhost(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid server message: j3j3h3j42");
    return;
  }
  data += tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 4; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_dns() {
  string req = "DNS";
  this->writeline(req + "\n", this->fd);
  return;
}
void irc::handle_replstatuline(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid server message: drcftg67y8");
    return;
  }
  data += tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 4; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_repllist(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid server message: 23ml4kjnrt");
    return;
  }
  for (int i = 3; i < tokens.size(); i++) {
    if (tokens[i][0] == ':') {
      tokens[i] = tokens[i].substr(1, tokens[i].length() - 1);
    }
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_replendlist(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 3) {
    this->print_error("invalid server message: 09cuodsifhkjb");
    return;
  }
  for (int i = 3; i < tokens.size(); i++) {
    if (tokens[i][0] == ':') {
      tokens[i] = tokens[i].substr(1, tokens[i].length() - 1);
    }
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_replstatsuptime(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 4) {
    this->print_error("invalid server message: kmjuhy76t");
    return;
  }
  data += tokens[3].substr(1, tokens[3].length() - 1);
  for (int i = 4; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::handle_replnotopic(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 5) {
    this->print_error("invalid server message:  0239roenw");
    return;
  }
  data += tokens[3] + " ";
  for (int i = 4; i < tokens.size(); i++) {
    if (tokens[i][0] == ':') {
      tokens[i] = tokens[i].substr(1, tokens[i].length() - 1);
    }
    data += " " + tokens[i];
  }
  this->add_history_line(tokens[3], data);
  return;
}
void irc::handle_replwho(vector<string> tokens) {
  string data = "-!- ";
  if (tokens.size() < 8) {
    this->print_error("invalid server message:  dkfej9r4");
    return;
  }
  for (int i = 7; i < tokens.size(); i++) {
    data += " " + tokens[i];
  }
  this->add_history_line("main", data);
  return;
}
void irc::parse_server_msg(string line) {
  string imm;
  vector<string> tokens;
  string target = "";
  string source = "";
  string message = "";
  string user = "";
  if (line.size() == 0) {
    return;
  }
  rtrim(line);
  stringstream ss(line);
  while (getline(ss, imm, ' ')) {
    tokens.push_back(imm);
  }
  if (tokens[0][0] == ':') {
    target = tokens[0];
    if (tokens.size() < 2) {
      return;
    }
    if (tokens[1] == "PRIVMSG") {
      this->handle_privmsg_response(tokens);
    } else if (tokens[1] == "JOIN") {
      this->handle_join_response(tokens);
    } else if (tokens[1] == "001" || tokens[1] == "002" || tokens[1] == "003" ||
               tokens[1] == "004") {
      this->handle_welcome_msg(tokens);
    } else if (tokens[1] == "005") {
      this->handle_lparams(tokens);
    } else if (tokens[1] == "020") {
      this->handle_plzwait(tokens);
    } else if (tokens[1] == "042") {
      this->handle_ircid(tokens);
    } else if (tokens[1] == "042") {
      this->handle_ircid(tokens);
    } else if (tokens[1] == "211") {
      this->handle_replstatslinkinfo(tokens);
    } else if (tokens[1] == "212") {
      this->handle_replstatscmds(tokens);
    } else if (tokens[1] == "219") {
      this->handle_replendofstats(tokens);
    } else if (tokens[1] == "242") {
      this->handle_replstatsuptime(tokens);
    } else if (tokens[1] == "249") {
      this->handle_replstatuline(tokens);
    } else if (tokens[1] == "251") {
      this->handle_luserlist(tokens);
    } else if (tokens[1] == "252") {
      this->handle_luserop(tokens);
    } else if (tokens[1] == "253") {
      this->handle_luserunknown(tokens);
    } else if (tokens[1] == "254") {
      this->handle_lchanlist(tokens);
    } else if (tokens[1] == "255") {
      this->handle_luserme(tokens);
    } else if (tokens[1] == "256") {
      this->handle_repladmin(tokens);
    } else if (tokens[1] == "257") {
      this->handle_repladminloc1(tokens);
    } else if (tokens[1] == "258") {
      this->handle_repladminloc2(tokens);
    } else if (tokens[1] == "259") {
      this->handle_repladminemail(tokens);
    } else if (tokens[1] == "265") {
      this->handle_localusers(tokens);
    } else if (tokens[1] == "266") {
      this->handle_globalusers(tokens);
    } else if (tokens[1] == "301") {
      this->handle_replaway(tokens);
    } else if (tokens[1] == "302") {
      this->handle_repluserhost(tokens);
    } else if (tokens[1] == "303") {
      this->handle_replison(tokens);
    } else if (tokens[1] == "305") {
      this->handle_replunaway(tokens);
    } else if (tokens[1] == "306") {
      this->handle_replnowaway(tokens);
    } else if (tokens[1] == "315") {
      this->handle_replendofwho(tokens);
    } else if (tokens[1] == "322") {
      this->handle_repllist(tokens);
    } else if (tokens[1] == "323") {
      this->handle_replendlist(tokens);
    } else if (tokens[1] == "331") {
      this->handle_replnotopic(tokens);
    } else if (tokens[1] == "332") {
      this->handle_repltopic(tokens);
    } else if (tokens[1] == "333") {
      this->handle_replpass(tokens);
    } else if (tokens[1] == "341") {
      this->handle_inviting(tokens);
    } else if (tokens[1] == "352") {
      this->handle_replwho(tokens);
    } else if (tokens[1] == "353") {
      this->handle_namereply(tokens);
    } else if (tokens[1] == "366") {
      this->handle_endofnames(tokens);
    } else if (tokens[1] == "371") {
      this->handle_replinfo(tokens);
    } else if (tokens[1] == "372") {
      this->handle_motd(tokens);
    } else if (tokens[1] == "374") {
      this->handle_endreplinfo(tokens);
    } else if (tokens[1] == "375") {
      this->handle_motdstart(tokens);
    } else if (tokens[1] == "376") {
      this->handle_endmotd(tokens);
    } else if (tokens[1] == "391") {
      this->handle_repltime(tokens);
    } else if (tokens[1] == "401") {
      this->handle_nosuchnick(tokens);
    } else if (tokens[1] == "403") {
      this->handle_nosuchchannel(tokens);
    } else if (tokens[1] == "421") {
      this->handle_unkcmd(tokens);
    } else if (tokens[1] == "433") {
      this->handle_nicknameinuse(tokens);
    } else if (tokens[1] == "442") {
      this->handle_replnotonchan(tokens);
    } else if (tokens[1] == "481") {
      this->handle_replnoperm(tokens);
    } else if (tokens[1] == "INVITE") {
      this->handle_invite_response(tokens);
    } else if (tokens[1] == "NOTICE") {
      this->handle_notice(tokens);
    } else if (tokens[1] == "NICK") {
      this->handle_newnick(tokens);
    } else if (tokens[1] == "QUIT") {
      this->handle_replquit(tokens);
    } else if (tokens[1] == "TOPIC") {
      this->handle_repltopicset(tokens);
    } else if (tokens[1] == "PART") {
      this->handle_part_response(tokens);
    } else {
      cout << "SERVER MESSAGE UNHANDLED: " << line << endl;
    }
  }
  return;
}
void irc::prompt(void) {
  string line;
  string imm;
  vector<string> tokens;
  fd_set read_fd_set;
  while (1) {
    tokens.clear();
    line.clear();
    FD_ZERO(&read_fd_set);
    FD_SET(fileno(stdin), &read_fd_set);
    if (this->connected) {
      FD_SET(this->fd, &read_fd_set);
    }
    if (select(FD_SETSIZE, &read_fd_set, NULL, NULL, NULL) < 0) {
      this->print_error("select() fail: " + string(strerror(errno)));
      exit(-1);
    }
    if (FD_ISSET(fileno(stdin), &read_fd_set)) {
      line = this->readline(fileno(stdin));
      stringstream ss(line);
      while (getline(ss, imm, ' ')) {
        tokens.push_back(imm);
      }
      if (tokens.size() <= 0) {
        cout << "[ERROR] stdin has an issue" << endl;
        exit(-1);
      }
      if (tokens[0] == "/quit") {
        if (this->connected) {
          this->handle_disconnect();
        }
        exit(0);
      } else if (tokens[0] == "/admin") {
        this->handle_admin(tokens);
      } else if (tokens[0] == "/away") {
        this->handle_away(tokens);
      } else if (tokens[0] == string("/connect")) {
        if (tokens.size() < 2) {
          this->print_error("Not enough parameters given");
          continue;
        }
        this->set_server(tokens[1]);
        this->conn();
      } else if (tokens[0] == "/disconnect") {
        this->handle_disconnect();
      } else if (tokens[0] == "/dns") {
        this->handle_dns();
      } else if (tokens[0] == "/info") {
        this->handle_info();
      } else if (tokens[0] == "/invite") {
        this->handle_invite(tokens);
      } else if (tokens[0] == "/ison") {
        this->handle_ison(tokens);
      } else if (tokens[0] == "/join") {
        this->handle_join(tokens);
      } else if (tokens[0] == "/list") {
        this->handle_list();
      } else if (tokens[0] == "/listchans") {
        this->list_channels();
      } else if (tokens[0] == "/log") {
        this->handle_log();
      } else if (tokens[0] == "/motd") {
        this->handle_cmdmotd(tokens);
      } else if (tokens[0] == "/msg") {
        this->handle_msg(tokens);
      } else if (tokens[0] == "/names") {
        this->handle_names(tokens);
      } else if (tokens[0] == "/nick") {
        this->handle_nick(tokens);
      } else if (tokens[0] == "/part") {
        this->handle_part(tokens);
      } else if (tokens[0] == "/printchan") {
        this->print_chan(tokens);
      } else if (tokens[0] == "/stats") {
        this->handle_stats(tokens);
      } else if (tokens[0] == "/time") {
        this->handle_time();
      } else if (tokens[0] == "/topic") {
        this->handle_topic(tokens);
      } else if (tokens[0] == "/userhost") {
        this->handle_userhost(tokens);
      } else if (tokens[0] == "/userip") {
        this->handle_userip(tokens);
      } else if (tokens[0] == "/watch") {
        this->handle_watch(tokens);
      } else if (tokens[0] == "/who") {
        this->handle_who(tokens);
      } else {
        this->print_error("unknown command: " + line);
      }
    } else if (this->connected && FD_ISSET(this->fd, &read_fd_set)) {
      line = this->readline(this->fd);
      if (line.find("PING") != string::npos) {
        this->handle_pong();
      } else {
        this->parse_server_msg(line);
      }
    }
  }
  return;
}
int main(int argc, char **argv) {
  irc client;
  char *nick = getenv("IRCNICK");
  if (nick == NULL) {
    nick = (char *)"chess";
  }
  client.set_nick(nick);
  client.prompt();
  return 0;
}