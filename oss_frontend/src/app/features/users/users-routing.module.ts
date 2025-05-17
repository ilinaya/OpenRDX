import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { UserGroupListComponent } from './components/user-group-list/user-group-list.component';
import { UserGroupFormComponent } from './components/user-group-form/user-group-form.component';

const routes: Routes = [
  {
    path: 'groups',
    children: [
      { path: '', component: UserGroupListComponent },
      { path: 'create', component: UserGroupFormComponent },
      { path: ':id/edit', component: UserGroupFormComponent }
    ]
  }
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule]
})
export class UsersRoutingModule { } 