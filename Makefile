setup:
	mkdir workspace
	cargo build
	mkdir workspace/.aequorea
	mkdir workspace/.aequorea/objects
	touch workspace/.aequorea/index

run:
	mkdir workspace/hoge
	touch workspace/fuga.py
	echo "import __hello__" > workspace/fuga.py
	touch workspace/hoge/fuga.rs
	echo "println!(\"hello\");" > workspace/hoge/fuga.rs
	cd workspace && ../target/debug/aequorea
