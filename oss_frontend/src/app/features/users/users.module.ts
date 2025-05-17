import { NgModule } from '@angular/core';
import { CommonModule, DatePipe } from '@angular/common';
import { ReactiveFormsModule, FormsModule } from '@angular/forms';
import { RouterModule } from '@angular/router';
import { UsersRoutingModule } from './users-routing.module';

import { UserGroupListComponent } from './components/user-group-list/user-group-list.component';
import { UserGroupFormComponent } from './components/user-group-form/user-group-form.component';

@NgModule({
  declarations: [
    UserGroupListComponent,
    UserGroupFormComponent
  ],
  imports: [
    CommonModule,
    ReactiveFormsModule,
    FormsModule,
    RouterModule,
    UsersRoutingModule
  ],
  providers: [
    DatePipe
  ]
})
export class UsersModule { }