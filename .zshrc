fastfetch

# Enable Powerlevel10k-style instant prompt if you ever use it, 
# but for Starship, we keep it simple.

# 1. Path Configuration (Critical for Rust apps)
export PATH="$HOME/.cargo/bin:$PATH"

# 2. History Configuration
HISTFILE=~/.zsh_history
HISTSIZE=10000
SAVEHIST=10000
setopt APPEND_HISTORY
setopt SHARE_HISTORY

# 3. Completion System
autoload -Uz compinit
compinit

# 4. Keybindings (Standard Emacs-style)
bindkey -e
bindkey '^[[3~' delete-char
bindkey '^[[1;5C' forward-word
bindkey '^[[1;5D' backward-word

# 5. Aliases
# Use 'bat' instead of 'cat' (since we installed bat)
if command -v bat &> /dev/null; then
    alias cat='bat --paging=never --style=plain'
    alias less='bat --paging=always'
fi

# Colorize grep and ls
alias grep='grep --color=auto'
alias ls='ls --color=auto'
alias ll='ls -lh'
alias la='ls -lha'

# Safety
alias cp='cp -i'
alias mv='mv -i'
alias rm='rm -i'

# 6. Initialize Starship (The Rust Prompt)
if command -v starship &> /dev/null; then
    eval "$(starship init zsh)"
fi
