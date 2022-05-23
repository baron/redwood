install:
	cargo install --path . --root /

install-zsh-completions:
	cp ./etc/zsh-completion/_redwood /usr/share/zsh/functions/Completion/Unix/
