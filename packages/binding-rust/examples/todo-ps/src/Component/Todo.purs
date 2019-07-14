module Component.Todo where

import Prelude

import Data.Maybe (Maybe(..), fromMaybe)
import Data.Array (snoc, filter, length, mapWithIndex, modifyAt)

import Halogen as H
import Halogen.HTML as HH
import Halogen.HTML.Events as HE
import Halogen.HTML.Properties as HP

import DOM.HTML.Indexed.InputType as IT
import DOM.Event.KeyboardEvent (key)

type Task =
  { name :: String
  , done :: Boolean
  }

type State =
  { tasks :: Array Task
  , numCompleted :: Int
  , newTaskName :: String
  }

initialState :: State
initialState =
  { tasks: []
  , numCompleted: 0
  , newTaskName: ""
  }

data Query a
  = UpdateNewTask String a
  | NewTask a
  | ToggleCompleted Int a
  | RemoveCompleted a

type Message = Void

component :: forall eff. H.Component HH.HTML Query Unit Message eff
component =
  H.component
    { initialState: const initialState
    , render
    , eval
    , receiver: const Nothing
    }
  where

  render :: State -> H.ComponentHTML Query
  render st =
    HH.div [ HP.class_ $ H.ClassName "container" ]
      [ HH.div [ HP.class_ $ H.ClassName "text-input-wrapper" ]
          [ HH.input
              [ HP.type_ IT.InputText
              , HP.class_ $ H.ClassName "text-input"
              , HP.autofocus true
              , HP.placeholder "new task"
              , HP.value st.newTaskName
              , HE.onValueInput (HE.input UpdateNewTask)
              , HE.onKeyDown \e -> case key e of
                  "Enter" -> Just (H.action NewTask)
                  _       -> Nothing
              ]
          ]
      , HH.div [ HP.class_ $ H.ClassName "task-list" ] $ mapWithIndex renderTask st.tasks
      , HH.div [ HP.class_ $ H.ClassName "footer" ]
          [ HH.div
              [ HP.class_ $ H.ClassName "btn-clear-tasks"
              , HE.onClick (HE.input_ RemoveCompleted)
              ]
              [ HH.text $ "Delete completed (" <> show st.numCompleted <> "/" <> show (length st.tasks) <> ")" ]
          ]
      ]

  renderTask i t =
    HH.div
      [ HP.class_ $ H.ClassName $ "task-item " <> checked
      , HE.onClick (HE.input_ $ ToggleCompleted i)
      ]
      [ HH.text t.name
      ]
    where checked = if t.done then "checked" else "unchecked"

  eval :: Query ~> H.ComponentDSL State Query Message eff
  eval = case _ of
    UpdateNewTask newTaskName next -> do
      H.modify (_ { newTaskName = newTaskName })
      pure next
    NewTask next -> do
      H.modify \st -> if st.newTaskName /= "" then st { tasks = st.tasks `snoc` { name: st.newTaskName, done: false }, newTaskName = "" } else st
      pure next
    ToggleCompleted i next -> do
      H.modify \st -> let newTasks = fromMaybe st.tasks $ modifyAt i (\t -> t { done = not t.done }) st.tasks in st { tasks = newTasks, numCompleted = length $ filter (\t -> t.done) newTasks }
      pure next
    RemoveCompleted next -> do
      H.modify \st -> st { tasks = filter (\t -> not t.done) st.tasks, numCompleted = 0 }
      pure next
