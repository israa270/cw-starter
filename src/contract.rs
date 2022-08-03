#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, Poll, POLLS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-starter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = msg.admin.unwrap_or(info.sender.to_string());
    let validated_admin = deps.api.addr_validate(&admin)?;
    let config = Config {
        admin: validated_admin.clone(),
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", validated_admin.to_string()))
    // let state = State {
    //     count: msg.count,
    //     owner: info.sender.clone(),
    // };
    // set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // STATE.save(deps.storage, &state)?;

    // Ok(Response::new()
    //     .add_attribute("method", "instantiate")
    //     .add_attribute("owner", info.sender)
    //     .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => try_increment(deps),
        ExecuteMsg::Reset { count } => try_reset(deps, info, count),
    }
}

fn execute_create_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: String,
    question: String,
    options: Vec<String>,
) -> Result<Response, ContractError> {
    if options.len() > 10 {
        return Err(ContractError::TooManyOptions {});
    }

    let mut opts: Vec<(String, u64)> = vec![];
    for option in options {
        opts.push((option, 0));
    }

    let poll = Poll {
        creator: info.sender,
        question,
        options: opts
    };

    POLLS.save(deps.storage, poll_id, &poll)?;

    Ok(Response::new())
}

pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}

pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count(deps: Deps) -> StdResult<GetCountResponse> {
    let state = CONFIG.load(deps.storage)?;
    Ok(GetCountResponse { count: state.count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, attr};
    use crate::contract::instantiate; // the contract instantiate function
    use crate::msg::InstantiateMsg; // our instantate method

      // Two fake addresses we will use to mock_info
      pub const ADDR1: &str = "addr1";
      pub const ADDR2: &str = "addr2";

      #[cfg_attr(not(feature = "library"), entry_point)]
            pub fn execute(
                _deps: DepsMut,
                _env: Env,
                _info: MessageInfo,
                _msg: ExecuteMsg,
            ) -> Result<Response, ContractError> {
                match msg {
                    ExecuteMsg::CreatePoll {
                        poll_id,
                        question,
                        options,
                    } => unimplemented!(),
                    ExecuteMsg::Vote { poll_id, vote } => unimplemented!(),
                }
            }

    #[test]
    fn test_instantiate() {
        // Mock the dependencies, must be mutable so we can pass it as a mutable, empty vector means our contract has no balance
        let mut deps = mock_dependencies();
        // Mock the contract environment, contains the block info, contract address, etc.
        let env = mock_env();
        // Mock the message info, ADDR1 will be the sender, the empty vec means we sent no funds.
        let info = mock_info(ADDR1, &vec![]);

        // Create a message where we (the sender) will be an admin
        let msg = InstantiateMsg { admin: None };
        // Call instantiate, unwrap to assert success
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![attr("action", "instantiate"), attr("admin", ADDR1)]
        )
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
