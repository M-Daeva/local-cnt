#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:local-cnt";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let initial_state = State {
        owner: info.sender,
        count: msg.count,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &initial_state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", initial_state.owner.to_string())
        .add_attribute("count", initial_state.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => increment(deps),
        ExecuteMsg::Set { count } => set(deps, count),
    }
}

pub fn increment(deps: DepsMut) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    state.count += 1;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "increment")
        .add_attribute("owner", state.owner.to_string())
        .add_attribute("count", state.count.to_string()))
}

pub fn set(deps: DepsMut, count: u8) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    state.count = count;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "set")
        .add_attribute("owner", state.owner.to_string())
        .add_attribute("count", state.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => query_state(deps),
    }
}

pub fn query_state(deps: Deps) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;

    to_binary(&CountResponse { count: state.count })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new()
        .add_attribute("method", "migrate")
        .add_attribute("version", CONTRACT_VERSION))
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::ContractError;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{attr, from_binary, Empty, Env, MessageInfo, OwnedDeps, Response};

    pub const ALICE_ADDR: &str = "juno1gjqnuhv52pd2a7ets2vhw9w9qa9knyhyqd4qeg";

    type Instance = (
        OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
        Env,
        MessageInfo,
        Result<Response, ContractError>,
    );

    fn get_instance(count: u8, addr: &str) -> Instance {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(addr, &[]);
        let msg = InstantiateMsg { count };

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg);
        (deps, env, info, res)
    }

    #[test]
    fn test_init() {
        let (_, _, _, res) = get_instance(42, ALICE_ADDR);

        assert_eq!(
            res.unwrap().attributes,
            vec![
                attr("method", "instantiate"),
                attr("owner", ALICE_ADDR.to_string()),
                attr("count", "42")
            ]
        )
    }

    #[test]
    fn test_increment() {
        let (mut deps, env, info, _) = get_instance(42, ALICE_ADDR);
        let msg = ExecuteMsg::Increment {};
        let inc_res = execute(deps.as_mut(), env, info, msg);

        assert_eq!(
            inc_res.unwrap().attributes,
            vec![
                attr("method", "increment"),
                attr("owner", ALICE_ADDR.to_string()),
                attr("count", "43")
            ]
        )
    }

    #[test]
    fn test_set() {
        let (mut deps, env, info, _) = get_instance(42, ALICE_ADDR);
        let msg = ExecuteMsg::Set { count: 45 };
        let set_res = execute(deps.as_mut(), env, info, msg);

        assert_eq!(
            set_res.unwrap().attributes,
            vec![
                attr("method", "set"),
                attr("owner", ALICE_ADDR.to_string()),
                attr("count", "45")
            ]
        )
    }

    #[test]
    fn test_query() {
        let (deps, env, _, _) = get_instance(42, ALICE_ADDR);
        let msg = QueryMsg::GetCount {};
        let bin = query(deps.as_ref(), env, msg).unwrap();
        let res = from_binary::<CountResponse>(&bin).unwrap();

        assert_eq!(res.count, 42);
    }
}
