import { expect } from "chai";
import { step } from "mocha-steps";
import { describeWithAcala, transfer } from "./util";

describeWithAcala("Acala RPC (Balance)", (context) => {
	let alice: Signer;
	let alice_stash: Signer;

	before("init wallets", async function () {
		[alice, alice_stash] = await context.provider.getWallets();
	});

	step("genesis balance is setup correctly", async function () {
		expect((await context.provider.getBalance(alice.getAddress())).toString()).to.equal("8999999986219144000");
		expect((await context.provider.getBalance(alice.getAddress(), "latest")).toString()).to.equal("8999999986219144000");

		expect((await context.provider.getBalance(alice.getAddress(), "latest")).toString())
			.to.equal((await context.provider.api.query.system.account(await alice.getSubstrateAddress())).data.free.toString());
	});

	step("balance to be updated after transfer", async function () {
		this.timeout(15000);

		expect((await context.provider.getBalance(alice.getAddress())).toString()).to.equal("8999999986219144000");
		expect((await context.provider.getBalance(alice_stash.getAddress())).toString()).to.equal("10100000986219149180");

		await transfer(context, await alice.getSubstrateAddress(), await alice_stash.getSubstrateAddress(), 1000);
		expect((await context.provider.getBalance(alice.getAddress())).toString()).to.equal("8999999969451542608");
		expect((await context.provider.getBalance(alice_stash.getAddress())).toString()).to.equal("10100000986219150180");
		expect((await context.provider.getBalance(alice.getAddress(), "latest")).toString()).to.equal("8999999969451542608");
		expect((await context.provider.getBalance(alice_stash.getAddress(), "earliest")).toString()).to.equal("10100001000000000000");
	});
});
