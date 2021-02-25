use crate::Result;
use substrate_subxt::staking::{LedgerStoreExt, Staking, StakingLedger};
use substrate_subxt::{Client, ClientBuilder, DefaultNodeRuntime, Runtime};
use substrate_subxt::system::System;
use substrate_subxt::sp_core::crypto::{Ss58Codec, AccountId32};
use std::{convert::TryFrom, vec};

pub struct StashAccount<T>(T);

impl<T> StashAccount<T> {
    fn raw(&self) -> &T {
        &self.0
    }
}

impl<T: Ss58Codec> TryFrom<String> for StashAccount<T> {
    type Error = anyhow::Error;

    fn try_from(val: String) -> Result<Self> {
        Ok(<Self as TryFrom<&str>>::try_from(&val)?)
    }
}

impl<'a, T: Ss58Codec> TryFrom<&'a str> for StashAccount<T> {
    type Error = anyhow::Error;

    fn try_from(val: &'a str) -> Result<Self> {
        Ok(StashAccount(T::from_ss58check(val).map_err(|err| anyhow!("failed to convert value to Runtime account address: {:?}", err))?))
    }
}

pub struct ChainData<R: Runtime> {
    client: Client<R>,
}

impl<R: Runtime + Staking> ChainData<R> {
    async fn new(hostname: &str) -> Result<ChainData<R>> {
        Ok(ChainData {
            client: ClientBuilder::<R>::new().set_url(hostname).build().await?,
        })
    }
    async fn fetch_staking_ledger(
        &self,
        account: R::AccountId,
        at: Option<R::Hash>,
    ) -> Result<Option<StakingLedger<R::AccountId, R::Balance>>> {
        self.client
            .ledger(account, at)
            .await
            .map_err(|err| err.into())
    }
    async fn fetch_staking_ledger_by_stash(
        &self,
        stash: &StashAccount<R::AccountId>,
        at: Option<R::Hash>,
    ) -> Result<Option<StakingLedger<R::AccountId, R::Balance>>> {
        let mut entries = self.client.ledger_iter(at).await?;

        while let Some((_, ledger)) = entries.next().await? {
            if &ledger.stash == stash.raw() {
                return Ok(Some(ledger))
            }
        }

        Ok(None)
    }
}

#[tokio::test]
async fn fetch_staking_ledger() {
    use substrate_subxt::KusamaRuntime;

    let targets = [
        AccountId32::from_ss58check("EX9uchmfeSqKTM7cMMg8DkH49XV8i4R7a7rqCn8btpZBHDP").unwrap(),
        AccountId32::from_ss58check("G1rrUNQSk7CjjEmLSGcpNu72tVtyzbWdUvgmSer9eBitXWf").unwrap(),
        AccountId32::from_ss58check("HgTtJusFEn2gmMmB5wmJDnMRXKD6dzqCpNR7a99kkQ7BNvX").unwrap(),
    ];

    let onchain = ChainData::<KusamaRuntime>::new("wss://kusama-rpc.polkadot.io").await.unwrap();
    let ledgers = onchain.fetch_staking_ledger_by_stash(&targets, None).await.unwrap();
    for ledger in ledgers {
        println!("\n\n>> {:?}", ledger);
    }
}
