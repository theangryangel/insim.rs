import React from 'react';
import { useContext, useReducer, useEffect } from 'react';
import ReactDOM from 'react-dom';
import Colourify from './components/colourify.jsx';
import { SSEProvider, SSEContext } from 'react-hooks-sse';
import { Tab } from '@headlessui/react';

import { Allotment } from "allotment";

function useSSE(
  eventNames,
  initialState,
  options
) {
  const {
    stateReducer = (_, action) => action.data,
    parser = (data) => JSON.parse(data),
    context = SSEContext,
  } = options || {};

  const source = useContext(context);
  const [state, dispatch] = useReducer(
    stateReducer,
    initialState
  );

  if (!source) {
    throw new Error(
      'Could not find an SSE context; You have to wrap useSSE() in a <SSEProvider>.'
    );
  }

  useEffect(() => {
    const listener = event => {
      const data = parser(event.data);

      dispatch({
        event,
        data,
      });
    };

    for (const eventName of eventNames) {
      source.addEventListener(eventName, listener);
    }

    return () => {
      for (const eventName of eventNames) {
        source.removeEventListener(eventName, listener);
      }
    };
  }, []);

  return state;
}

function Overlay() {
  const state = useSSE(['MessageOut', 'Lap'], {
    messages: [],
    laps: [],
  }, {
    stateReducer(state, action) {

      if (action.data.type == "MessageOut") {
        return {
          ...state, messages: [action.data.msg].concat(state.messages).slice(0, 30)
        };
      }

      if (action.data.type == "Lap") {
        return {
          ...state,
          laps: state.laps.concat([action.data])
        }
      }
    }
  });

  return (
    <div className="border border-red-900">


    <Tab.Group>
      <Tab.List>
        <Tab>Messages</Tab>
        <Tab>Laps</Tab>
        <Tab>Positions</Tab>
      </Tab.List>
      <Tab.Panels>
        <Tab.Panel>
          <ul>
            {state.messages.map((m) => {
              return (<li>{m}</li>);
            })}
          </ul>

        </Tab.Panel>
        <Tab.Panel>
          <ul>
          {state.laps.map((l) => {
            return (<li>{JSON.stringify(l)}</li>);
          })}
          </ul>
        </Tab.Panel>
        <Tab.Panel>Content 3</Tab.Panel>
      </Tab.Panels>
    </Tab.Group>



    </div>  
  );
}

function Map(props) {
  const state = useSSE(['MultiCarInfo', 'Npl', 'Pll'], {
    data: {},
  }, {
    stateReducer(state, action) {

      if (action.event.type == "MultiCarInfo") {
        return {
          data: action.data.info.reduce((prev, cur) => {
            return Object.assign(prev, {
              [cur.plid]: cur,
            })
          }, state.data)
        };
      }

      if (action.event.type == "Npl") {
        console.log(action.data);
      }

      if (["Pll", "Plp"].includes(action.event.type)) {
        let data = Object.fromEntries(
          Object.entries(state.data).filter(
            ([key]) => key == action.data.plid
          )
        );

        return {
          data: data
        }
      }

    }
  });

  return (
    <div>
    <span className="text-red-900">Map content here.</span>

    {props.children}

    <table className="table-auto w-50">
    <thead>
    <tr>
    <th>#</th>
    <th>Lap</th>
    <th>Position</th>
    <th>Coordinates</th>
    <th>Speed</th>
    </tr>
    </thead>
    <tbody>

    {Object.entries(state.data).map(([idx, p]) => {

      return (<tr key={p.plid}>
        <td>{p.plid}</td>
        <td>{p.lap}</td>
        <td>{p.position}</td>
        <td><code>({p.x}, {p.y}, {p.z})</code></td>
        <td>{p.speed}</td>
        </tr>);

    } )}

    </tbody>
    </table>

    </div>
  );
}

ReactDOM.render(
  <SSEProvider endpoint="sse">
  <Allotment>
    <Map/>
    <Overlay/>
  </Allotment>
  </SSEProvider>,
  document.getElementById('root')
);
